pipeline {

  environment {
    GIT_COMMIT_ID = sh (script: 'git rev-parse --short HEAD ${GIT_COMMIT}', returnStdout: true).trim()
    GIT_COMMIT_MSG = sh (script: 'git log -1 --pretty=%B ${GIT_COMMIT}', returnStdout: true).trim()
  }

  agent {
    kubernetes {
      yaml """
apiVersion: v1
kind: Pod
spec:
  containers:
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
    - name: docker
      hostPath:
        path: /var/run/docker.sock
"""
    }
  }

  stages {

    stage('build and push docker image') {
      
      environment {
        REGISTRY = credentials('c5425e91-91d8-4084-8011-82c6497cd40a')
        REGISTRY_URL = credentials('automata-docker-registry-url')
        REGISTRY_BASE_REPO = credentials('automata-docker-registry-base-repo')

        DOCKER_HUB = credentials('automata-docker-hub')
        DOCKER_HUB_BASE_REPO = "atactr"
      }

      steps {
        container('image-builder') {
          script {

            def dockerHubTag = "${env.DOCKER_HUB_BASE_REPO}/automata:${env.GIT_COMMIT_ID}"
            def registryTag = "${env.REGISTRY_URL}/${env.REGISTRY_BASE_REPO}/automata:${env.GIT_COMMIT_ID}"

            echo "build and tag image"
            sh "docker build -t ${dockerHubTag} ."
            sh "docker tag ${dockerHubTag} ${registryTag}"

            echo "push image"
            sh "docker login -u $DOCKER_HUB_USR -p $DOCKER_HUB_PSW"
            sh "docker push ${dockerHubTag}"

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
