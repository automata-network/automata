pipeline {

  environment {
    PROFILE = "release"

    GIT_COMMIT_ID = sh (script: 'git rev-parse --short HEAD', returnStdout: true).trim()
    GIT_COMMIT_MSG = sh (script: 'git log -1 --pretty=%B', returnStdout: true).trim()
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
      
      steps {
        container('rust') {
          script {
            sh "ln -s /cache/target ./target"
            sh "cargo build --${env.PROFILE} --bin automata"
            sh "cp ./target/${PROFILE}/automata ."
          }
        }
      }
    }

    stage('build and push docker image') {
      
      environment {
        REGISTRY = credentials('c5425e91-91d8-4084-8011-82c6497cd40a')
        REGISTRY_URL = credentials('automata-docker-registry-url')
        REGISTRY_BASE_REPO = credentials('automata-docker-registry-base-repo')
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

    stage('deploy') {
      environment {
        K8S_MASTER_IP = credentials('jiaxing-k8s-master-ip')
        K8S_MASTER = credentials('381816aa-abe9-4a66-8842-5f141dff42b4')
      }
      steps {
        container('sshpass') {
            // todo
        }
      }
    }
  }
}
