pipeline {
agent any
options {
      timeout(time:2, unit: 'HOURS')
    }
    stages {
        //     stage('Build the WebAssembly binary') {
        //     steps {
        //         sh './scripts/build.sh'
        //     }
        // }
        stage('Build all native code') {
            steps {
                sh 'cargo build --release --jobs 8'
            }
        }
        stage('Test') {
            steps {
                sh 'cargo test'
            }
        }
        // stage('Master Build') {
        //     when {
        //         branch 'master'
        //     }
        //     steps {
        //         sh 'docker build -t dnatest -f ./scripts/Docker/Dockerfile'
        //     }
        // }
    }
    post {
        success {
            googlechatnotification url: 'https://chat.googleapis.com/v1/spaces/AAAAwYIn96U/messages?key=AIzaSyDdI0hCZtE6vySjMm-WEfRq3CPzqKqqsHI&token=N2-GhV04rRI9qZVDgG0gdve3XeUdtMng8jOf-aPdcKc%3D&threadKey=jenkins', message: '<https://bitbucket.org/apigarage-core/metaverse-dna/pull-requests/${CHANGE_ID}|${JOB_NAME}> is ${BUILD_STATUS} by ${GIT_AUTHOR_NAME} *SUCCESS*.'
        }
        failure {
            googlechatnotification url: 'https://chat.googleapis.com/v1/spaces/AAAAwYIn96U/messages?key=AIzaSyDdI0hCZtE6vySjMm-WEfRq3CPzqKqqsHI&token=N2-GhV04rRI9qZVDgG0gdve3XeUdtMng8jOf-aPdcKc%3D&threadKey=jenkins', message: '<https://bitbucket.org/apigarage-core/metaverse-dna/pull-requests/${CHANGE_ID}|${JOB_NAME}> is ${BUILD_STATUS} by ${GIT_AUTHOR_NAME} *FAIL*.'
        }
        aborted {
            googlechatnotification url: 'https://chat.googleapis.com/v1/spaces/AAAAwYIn96U/messages?key=AIzaSyDdI0hCZtE6vySjMm-WEfRq3CPzqKqqsHI&token=N2-GhV04rRI9qZVDgG0gdve3XeUdtMng8jOf-aPdcKc%3D&threadKey=jenkins', message: '<https://bitbucket.org/apigarage-core/metaverse-dna/pull-requests/${CHANGE_ID}|${JOB_NAME}> is ${BUILD_STATUS} by ${GIT_AUTHOR_NAME} *ABORTED*.'
        }

    }
}