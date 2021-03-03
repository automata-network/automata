pipeline {

  environment {
    PROFILE = "release"

    GIT_COMMIT_ID = sh (script: 'git rev-parse --short HEAD', returnStdout: true).trim()
    GIT_COMMIT_MSG = sh (script: 'git log -1 --pretty=%B', returnStdout: true).trim()

    REGISTRY_URL = credentials('automata-docker-registry-url')
    REGISTRY_BASE_REPO = credentials('automata-docker-registry-base-repo')
  }
  parameters {
    choice(
      description: 'Choose your operation',
      name: 'operation',
      choices: ['deploy', 'upgrade', 'upgrade-skip-compile', 'delete']
    )
    text(
      description: 'Empty means insertKey, not empty will sync extern chain data',
      name: 'syncNodes'
    )
    choice(
      description: 'Choose the number of light nodes to run',
      name: 'lightCount',
      choices: ['', '0', '1', '2', '3', '4']
    )
    booleanParam(
      description: "Keep chain's data after delete (Only effective when deploying)",
      name: 'keepDataAfterDelete',
      defaultValue: true
    )
  }
  agent {
    kubernetes {
      yaml """
apiVersion: v1
kind: Pod
spec:
  containers:
    - name: rust
      image: chansonchan/automata-builder:nightly-2020-10-25
      command:
        - cat
      tty: true
      volumeMounts:
        - mountPath: /cache/target
          name: cache
          subPath: ${env.JOB_NAME}
        - mountPath: /usr/local/cargo/registry/index
          name: cache
          subPath: cargo/registry/index
        - mountPath: /usr/local/cargo/registry/cache
          name: cache
          subPath: cargo/registry/cache
        - mountPath: /usr/local/cargo/git/db
          name: cache
          subPath: cargo/git/db
    - name: image-builder
      image: docker:latest
      command:
        - cat
      tty: true
      volumeMounts:
        - mountPath: /var/run/docker.sock
          name: docker
    - name: sshpass
      image: chansonchan/sshpass
      command:
        - cat
      tty: true
  volumes:
    - name: cache
      persistentVolumeClaim:
        claimName: cache
    - name: docker
      hostPath:
        path: /var/run/docker.sock
"""
    }
  }

  stages {

    stage('compile') {
      when {
        beforeAgent true
        expression { params.operation ==~ /(deploy|upgrade)/ }
      }

      steps {
        container('rust') {
          script {
            echo "${GIT_COMMIT_MSG}"
            sh "ln -s /cache/target ./target"
            sh "cargo build --${env.PROFILE} --bin automata"
            sh "cp ./target/${PROFILE}/automata ."
          }
        }
      }
    }

    stage('build and push docker image') {
      when {
        beforeAgent true
        expression { params.operation ==~ /(deploy|upgrade)/ }
      }

      environment {
        REGISTRY = credentials('c5425e91-91d8-4084-8011-82c6497cd40a')
      }

      steps {
        container('image-builder') {
          script {
            def registryTag = "${env.REGISTRY_URL}/${env.REGISTRY_BASE_REPO}/automata:${env.GIT_COMMIT_ID}"

            echo "build and tag image"
            sh "docker build -t ${registryTag} -f .jenkins/AppDockerfile ."

            echo "push image"
            sh "docker login ${env.REGISTRY_URL} -u $REGISTRY_USR -p $REGISTRY_PSW"
            sh "docker push ${registryTag}"
          }
        }
      }
    }

    stage('operate') {
      environment {
        K8S_MASTER_IP = credentials('jiaxing-k8s-master-ip')
        K8S_MASTER = credentials('381816aa-abe9-4a66-8842-5f141dff42b4')

        STORAGE_CLASS_NAME = credentials('automata-storageClassName')
        IMAGE_PULL_SECRETS = credentials('automata-imagePullSecrets')
        CHAIN_KEY_SECRET = credentials('automata-chainKeySecret')
        NAMESPACE = credentials('automata-namespace')
      }
      steps {
        container('sshpass') {
          script {

            if (params.operation == "delete") {
              sh "sshpass -p $K8S_MASTER_PSW ssh -T -o StrictHostKeyChecking=no $K8S_MASTER_USR@$K8S_MASTER_IP" +
                      " \"helm uninstall -n ${env.NAMESPACE} ata\""
            } else {

              def operate, additionalSet = ""
              if (params.operation == "deploy") {
                operate = "install"

                if (params.keepDataAfterDelete != null) {
                  additionalSet += "--set chain.keepData=${params.keepDataAfterDelete} "
                }
              } else {
                operate = "upgrade"
              }

              if (params.lightCount != '') {
                additionalSet += "--set chain.lightCount=${params.lightCount} "
              }

              def syncNodes = "\"{", first = true
              for (item in params.syncNodes.tokenize('\n')) {
                if (first) {
                  syncNodes += item
                  first = false
                } else {
                  syncNodes += ", ${item}"
                }
              }
              syncNodes += "}\""
              
              if (!first) {
                additionalSet += "--set chain.syncNodes=${syncNodes} "
              }

              sh "sshpass -p $K8S_MASTER_PSW scp -o StrictHostKeyChecking=no -r .jenkins/automata-chart $K8S_MASTER_USR@$K8S_MASTER_IP:/tmp/automata-chart"

              sh "sshpass -p $K8S_MASTER_PSW ssh -T -o StrictHostKeyChecking=no $K8S_MASTER_USR@$K8S_MASTER_IP" +
                      " \"helm ${operate} --atomic -n ${env.NAMESPACE} ${additionalSet}" +
                      "--set storageClassName=${env.STORAGE_CLASS_NAME} " +
                      "--set image=${env.REGISTRY_URL}/${env.REGISTRY_BASE_REPO}/automata:${env.GIT_COMMIT_ID} " +
                      "--set imagePullSecrets=${env.IMAGE_PULL_SECRETS} " +
                      "--set chain.keySecret=${env.CHAIN_KEY_SECRET} " +
                      "ata /tmp/automata-chart && rm -rf /tmp/automata-chart\""
            }
          }
        }
      }
    }
  }
}
