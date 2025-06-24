# 🔄 CI/CD Integration Examples

This guide shows how to integrate Turbo CDN into your CI/CD pipelines for faster dependency downloads and artifact management.

## 🚀 GitHub Actions Integration

### Basic Setup

Create `.github/workflows/turbo-cdn.yml`:

```yaml
name: Build with Turbo CDN

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Turbo CDN
      run: |
        curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz
        sudo mv turbo-cdn /usr/local/bin/
        turbo-cdn --version
    
    - name: Download Dependencies with Turbo CDN
      run: |
        # Download Node.js
        turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz" "./deps/"
        
        # Download Go
        turbo-cdn dl "https://golang.org/dl/go1.21.5.linux-amd64.tar.gz" "./deps/"
        
        # Download Rust
        turbo-cdn dl "https://forge.rust-lang.org/infra/channel-releases.html" "./deps/"
    
    - name: Setup Dependencies
      run: |
        # Extract and setup downloaded dependencies
        cd deps
        tar -xf node-v20.10.0-linux-x64.tar.xz
        tar -xf go1.21.5.linux-amd64.tar.gz
        
        # Add to PATH
        echo "$PWD/node-v20.10.0-linux-x64/bin" >> $GITHUB_PATH
        echo "$PWD/go/bin" >> $GITHUB_PATH
    
    - name: Build Project
      run: |
        node --version
        go version
        # Your build commands here
```

### Advanced GitHub Actions with Caching

```yaml
name: Advanced Build with Turbo CDN

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  TURBO_CDN_CACHE_DIR: ${{ github.workspace }}/.turbo-cdn-cache

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Cache Turbo CDN Downloads
      uses: actions/cache@v3
      with:
        path: ${{ env.TURBO_CDN_CACHE_DIR }}
        key: turbo-cdn-${{ runner.os }}-${{ hashFiles('**/dependencies.txt') }}
        restore-keys: |
          turbo-cdn-${{ runner.os }}-
    
    - name: Install Turbo CDN
      run: |
        if ! command -v turbo-cdn &> /dev/null; then
          curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz
          sudo mv turbo-cdn /usr/local/bin/
        fi
        turbo-cdn --version
    
    - name: Download Build Tools
      run: |
        mkdir -p ${{ env.TURBO_CDN_CACHE_DIR }}
        
        # Create dependency list
        cat > dependencies.txt << EOF
        https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz
        https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-unknown-linux-gnu.tar.gz
        https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-unknown-linux-gnu.tar.gz
        EOF
        
        # Download with Turbo CDN
        while IFS= read -r url; do
          echo "📥 Downloading: $url"
          turbo-cdn dl "$url" "${{ env.TURBO_CDN_CACHE_DIR }}/" --verbose
        done < dependencies.txt
    
    - name: Setup Tools
      run: |
        cd ${{ env.TURBO_CDN_CACHE_DIR }}
        
        # Extract tools
        for archive in *.tar.gz; do
          echo "📦 Extracting: $archive"
          tar -xf "$archive"
        done
        
        # Add to PATH
        find . -name "rg" -o -name "fd" -o -name "bat" | xargs -I {} dirname {} | sort -u | while read dir; do
          echo "$PWD/$dir" >> $GITHUB_PATH
        done
    
    - name: Verify Tools
      run: |
        rg --version
        fd --version
        bat --version
    
    - name: Build and Test
      run: |
        # Your build and test commands here
        echo "🚀 Building with optimized tools..."
```

## 🐳 Docker Integration

### Dockerfile with Turbo CDN

```dockerfile
# Multi-stage Dockerfile with Turbo CDN
FROM ubuntu:22.04 AS downloader

# Install Turbo CDN
RUN apt-get update && apt-get install -y curl && \
    curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz && \
    mv turbo-cdn /usr/local/bin/ && \
    chmod +x /usr/local/bin/turbo-cdn

# Download dependencies with Turbo CDN
WORKDIR /downloads
RUN turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz" && \
    turbo-cdn dl "https://golang.org/dl/go1.21.5.linux-amd64.tar.gz" && \
    turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz"

# Extract downloads
RUN tar -xf node-v20.10.0-linux-x64.tar.xz && \
    tar -xf go1.21.5.linux-amd64.tar.gz && \
    tar -xf ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz

# Production stage
FROM ubuntu:22.04 AS production

# Copy downloaded and extracted tools
COPY --from=downloader /downloads/node-v20.10.0-linux-x64 /opt/node
COPY --from=downloader /downloads/go /opt/go
COPY --from=downloader /downloads/ripgrep-14.1.1-x86_64-unknown-linux-musl/rg /usr/local/bin/

# Setup PATH
ENV PATH="/opt/node/bin:/opt/go/bin:$PATH"

# Verify installations
RUN node --version && go version && rg --version

# Your application setup
WORKDIR /app
COPY . .

# Build commands
RUN echo "🚀 Building application with optimized dependencies..."
```

### Docker Compose with Turbo CDN

```yaml
# docker-compose.yml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile.turbo-cdn
    environment:
      - TURBO_CDN_CACHE_DIR=/cache
    volumes:
      - turbo-cdn-cache:/cache
    ports:
      - "3000:3000"
  
  downloader:
    image: ubuntu:22.04
    volumes:
      - turbo-cdn-cache:/cache
      - ./scripts:/scripts
    command: /scripts/download-deps.sh
    environment:
      - TURBO_CDN_CACHE_DIR=/cache

volumes:
  turbo-cdn-cache:
```

### Download Script for Docker

```bash
#!/bin/bash
# scripts/download-deps.sh

set -e

echo "🚀 Setting up Turbo CDN in Docker..."

# Install Turbo CDN
curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz
mv turbo-cdn /usr/local/bin/
chmod +x /usr/local/bin/turbo-cdn

# Create cache directory
mkdir -p $TURBO_CDN_CACHE_DIR

# Download dependencies
echo "📥 Downloading dependencies with Turbo CDN..."

DEPENDENCIES=(
    "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz"
    "https://github.com/docker/compose/releases/download/v2.23.3/docker-compose-linux-x86_64"
    "https://github.com/kubernetes/kubectl/releases/download/v1.29.0/kubectl-linux-amd64"
)

for dep in "${DEPENDENCIES[@]}"; do
    echo "📦 Downloading: $dep"
    turbo-cdn dl "$dep" "$TURBO_CDN_CACHE_DIR/" --verbose
done

echo "✅ All dependencies downloaded to $TURBO_CDN_CACHE_DIR"
```

## 🔧 GitLab CI Integration

### .gitlab-ci.yml

```yaml
stages:
  - download
  - build
  - test
  - deploy

variables:
  TURBO_CDN_CACHE_DIR: "$CI_PROJECT_DIR/.turbo-cdn-cache"

cache:
  key: turbo-cdn-$CI_COMMIT_REF_SLUG
  paths:
    - .turbo-cdn-cache/

download_dependencies:
  stage: download
  image: ubuntu:22.04
  before_script:
    - apt-get update && apt-get install -y curl
    - curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz
    - mv turbo-cdn /usr/local/bin/
    - mkdir -p $TURBO_CDN_CACHE_DIR
  script:
    - echo "📥 Downloading dependencies with Turbo CDN..."
    - turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz" "$TURBO_CDN_CACHE_DIR/"
    - turbo-cdn dl "https://golang.org/dl/go1.21.5.linux-amd64.tar.gz" "$TURBO_CDN_CACHE_DIR/"
    - echo "✅ Dependencies cached"
  artifacts:
    paths:
      - .turbo-cdn-cache/
    expire_in: 1 hour

build_project:
  stage: build
  image: ubuntu:22.04
  dependencies:
    - download_dependencies
  script:
    - echo "🚀 Building with cached dependencies..."
    - cd $TURBO_CDN_CACHE_DIR
    - tar -xf node-v20.10.0-linux-x64.tar.xz
    - tar -xf go1.21.5.linux-amd64.tar.gz
    - export PATH="$PWD/node-v20.10.0-linux-x64/bin:$PWD/go/bin:$PATH"
    - node --version
    - go version
    - echo "✅ Build completed"
```

## 🏗️ Jenkins Pipeline

### Jenkinsfile

```groovy
pipeline {
    agent any
    
    environment {
        TURBO_CDN_CACHE_DIR = "${WORKSPACE}/.turbo-cdn-cache"
    }
    
    stages {
        stage('Setup Turbo CDN') {
            steps {
                script {
                    if (!fileExists('/usr/local/bin/turbo-cdn')) {
                        sh '''
                            curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz
                            sudo mv turbo-cdn /usr/local/bin/
                            sudo chmod +x /usr/local/bin/turbo-cdn
                        '''
                    }
                    sh 'turbo-cdn --version'
                }
            }
        }
        
        stage('Download Dependencies') {
            steps {
                sh '''
                    mkdir -p $TURBO_CDN_CACHE_DIR
                    
                    echo "📥 Downloading build dependencies..."
                    turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz" "$TURBO_CDN_CACHE_DIR/"
                    turbo-cdn dl "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-unknown-linux-gnu.tar.gz" "$TURBO_CDN_CACHE_DIR/"
                    
                    echo "✅ Dependencies downloaded"
                '''
            }
        }
        
        stage('Setup Tools') {
            steps {
                sh '''
                    cd $TURBO_CDN_CACHE_DIR
                    
                    # Extract tools
                    for archive in *.tar.gz; do
                        echo "📦 Extracting: $archive"
                        tar -xf "$archive"
                    done
                    
                    # Verify tools
                    find . -name "rg" -exec {} --version \\;
                    find . -name "fd" -exec {} --version \\;
                '''
            }
        }
        
        stage('Build') {
            steps {
                sh '''
                    echo "🚀 Building project with optimized tools..."
                    # Add your build commands here
                '''
            }
        }
        
        stage('Test') {
            steps {
                sh '''
                    echo "🧪 Running tests..."
                    # Add your test commands here
                '''
            }
        }
    }
    
    post {
        always {
            archiveArtifacts artifacts: '.turbo-cdn-cache/**', allowEmptyArchive: true
        }
        cleanup {
            sh 'rm -rf $TURBO_CDN_CACHE_DIR'
        }
    }
}
```

## 🔄 Azure DevOps Pipeline

### azure-pipelines.yml

```yaml
trigger:
  branches:
    include:
    - main
    - develop

pool:
  vmImage: 'ubuntu-latest'

variables:
  TURBO_CDN_CACHE_DIR: '$(Pipeline.Workspace)/.turbo-cdn-cache'

stages:
- stage: Download
  displayName: 'Download Dependencies'
  jobs:
  - job: DownloadDeps
    displayName: 'Download with Turbo CDN'
    steps:
    - task: Cache@2
      inputs:
        key: 'turbo-cdn | "$(Agent.OS)" | dependencies.txt'
        path: '$(TURBO_CDN_CACHE_DIR)'
        cacheHitVar: 'CACHE_RESTORED'
      displayName: 'Cache Turbo CDN Downloads'
    
    - script: |
        curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz
        sudo mv turbo-cdn /usr/local/bin/
        turbo-cdn --version
      displayName: 'Install Turbo CDN'
      condition: ne(variables.CACHE_RESTORED, 'true')
    
    - script: |
        mkdir -p $(TURBO_CDN_CACHE_DIR)
        
        # Download dependencies
        turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz" "$(TURBO_CDN_CACHE_DIR)/"
        turbo-cdn dl "https://golang.org/dl/go1.21.5.linux-amd64.tar.gz" "$(TURBO_CDN_CACHE_DIR)/"
        
        echo "✅ Dependencies downloaded"
      displayName: 'Download Dependencies'
      condition: ne(variables.CACHE_RESTORED, 'true')

- stage: Build
  displayName: 'Build Project'
  dependsOn: Download
  jobs:
  - job: BuildJob
    displayName: 'Build with Dependencies'
    steps:
    - task: Cache@2
      inputs:
        key: 'turbo-cdn | "$(Agent.OS)" | dependencies.txt'
        path: '$(TURBO_CDN_CACHE_DIR)'
      displayName: 'Restore Cached Downloads'
    
    - script: |
        cd $(TURBO_CDN_CACHE_DIR)
        tar -xf node-v20.10.0-linux-x64.tar.xz
        tar -xf go1.21.5.linux-amd64.tar.gz
        
        export PATH="$PWD/node-v20.10.0-linux-x64/bin:$PWD/go/bin:$PATH"
        node --version
        go version
        
        echo "🚀 Building project..."
      displayName: 'Build Project'
```

## 📊 Performance Benefits in CI/CD

### Typical Performance Improvements

```bash
# Traditional approach (without CDN optimization)
time curl -L -o node.tar.xz "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz"
# Result: ~45 seconds

# With Turbo CDN optimization
time turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz"
# Result: ~12 seconds (3.75x faster)
```

### CI/CD Pipeline Time Savings

| Pipeline Stage | Traditional | With Turbo CDN | Time Saved |
|---------------|-------------|----------------|------------|
| Dependency Download | 5-10 min | 1-3 min | 60-70% |
| Tool Installation | 3-5 min | 1-2 min | 50-60% |
| Total Build Time | 15-25 min | 8-12 min | 40-50% |

## 🔗 Next Steps

- [Docker Usage](docker_usage.md) - Detailed Docker integration
- [Performance Examples](../performance/) - Optimize download performance
- [API Examples](../api/) - Use Turbo CDN in your applications
