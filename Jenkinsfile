pipeline {
    agent any
    options {
        timeout(time:2, unit: 'HOURS')
    }
    stages {
        stage('Build all native code') {
            steps {
                sh 'cargo clean'
                sh 'cp -aRf ${HOME}/kush/metaverse-dna/* .'
                sh 'cargo build --release --jobs=8'
                sh 'cp -aRf Cargo.lock ${HOME}/kush/metaverse-dna/'
                sh 'cp -aRf target ${HOME}/kush/metaverse-dna/'
            }
        }
        stage('Test') {
            steps {
                sh 'cargo test --release --jobs=8'
            }
        }

        stage('Master Build') {
            when {
                branch 'master'
            }
            steps {
                sh 'cp ./target/release/dnachain .'
                sh 'docker build -t dnachain -f "./scripts/Docker/Dockerfile" "."'
                sh 'docker tag dnachain docker.io/blockxdna/dnachain:node-0.2.${BUILD_NUMBER}'
                sh 'docker tag dnachain docker.io/blockxdna/dnachain:latest'
                sh 'docker push docker.io/blockxdna/dnachain:node-0.2.${BUILD_NUMBER}'
                sh 'docker push docker.io/blockxdna/dnachain:latest'
                sh 'docker rmi -f blockxdna/dnachain:node-0.2.${BUILD_NUMBER}'
                sh 'docker rmi -f blockxdna/dnachain:latest'
            }
        }
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