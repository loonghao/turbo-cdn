# 🐳 Docker Integration Examples

This guide demonstrates how to integrate Turbo CDN into Docker workflows for faster container builds and dependency management.

## 🚀 Basic Docker Integration

### Simple Dockerfile with Turbo CDN

```dockerfile
FROM ubuntu:22.04

# Install Turbo CDN
RUN apt-get update && apt-get install -y curl && \
    curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz && \
    mv turbo-cdn /usr/local/bin/ && \
    chmod +x /usr/local/bin/turbo-cdn && \
    rm -rf /var/lib/apt/lists/*

# Download dependencies with Turbo CDN
WORKDIR /tmp
RUN turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz" && \
    tar -xf node-v20.10.0-linux-x64.tar.xz && \
    mv node-v20.10.0-linux-x64 /opt/node && \
    rm node-v20.10.0-linux-x64.tar.xz

# Setup PATH
ENV PATH="/opt/node/bin:$PATH"

# Verify installation
RUN node --version && npm --version

WORKDIR /app
COPY . .

# Install application dependencies
RUN npm install

EXPOSE 3000
CMD ["npm", "start"]
```

## 🏗️ Multi-Stage Docker Builds

### Optimized Multi-Stage Build

```dockerfile
# Stage 1: Download dependencies with Turbo CDN
FROM ubuntu:22.04 AS downloader

# Install Turbo CDN
RUN apt-get update && apt-get install -y curl && \
    curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz && \
    mv turbo-cdn /usr/local/bin/ && \
    chmod +x /usr/local/bin/turbo-cdn

# Create download directory
WORKDIR /downloads

# Download all dependencies in parallel
RUN echo "📥 Downloading dependencies with Turbo CDN..." && \
    turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz" & \
    turbo-cdn dl "https://golang.org/dl/go1.21.5.linux-amd64.tar.gz" & \
    turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz" & \
    wait

# Extract downloads
RUN echo "📦 Extracting downloads..." && \
    tar -xf node-v20.10.0-linux-x64.tar.xz && \
    tar -xf go1.21.5.linux-amd64.tar.gz && \
    tar -xf ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz

# Stage 2: Build environment
FROM ubuntu:22.04 AS builder

# Copy extracted dependencies from downloader stage
COPY --from=downloader /downloads/node-v20.10.0-linux-x64 /opt/node
COPY --from=downloader /downloads/go /opt/go
COPY --from=downloader /downloads/ripgrep-14.1.1-x86_64-unknown-linux-musl/rg /usr/local/bin/

# Setup environment
ENV PATH="/opt/node/bin:/opt/go/bin:$PATH"

# Verify tools
RUN node --version && go version && rg --version

# Copy source code
WORKDIR /app
COPY . .

# Build application
RUN echo "🚀 Building application..." && \
    npm install && \
    npm run build

# Stage 3: Production runtime
FROM ubuntu:22.04 AS production

# Install only runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy Node.js runtime
COPY --from=builder /opt/node /opt/node

# Copy built application
COPY --from=builder /app/dist /app
COPY --from=builder /app/package.json /app/

# Setup environment
ENV PATH="/opt/node/bin:$PATH"
ENV NODE_ENV=production

WORKDIR /app

# Install production dependencies only
RUN npm install --only=production

EXPOSE 3000
CMD ["node", "index.js"]
```

## 🔄 Docker Compose with Turbo CDN

### Development Environment

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile.dev
      args:
        - TURBO_CDN_CACHE_DIR=/cache
    volumes:
      - .:/app
      - turbo-cdn-cache:/cache
      - node_modules:/app/node_modules
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=development
      - TURBO_CDN_CACHE_DIR=/cache
    depends_on:
      - downloader

  downloader:
    build:
      context: .
      dockerfile: Dockerfile.downloader
    volumes:
      - turbo-cdn-cache:/cache
    environment:
      - TURBO_CDN_CACHE_DIR=/cache

  database:
    image: postgres:15
    environment:
      - POSTGRES_DB=myapp
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  turbo-cdn-cache:
  node_modules:
  postgres_data:
```

### Dockerfile.downloader

```dockerfile
FROM ubuntu:22.04

# Install Turbo CDN
RUN apt-get update && apt-get install -y curl && \
    curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz && \
    mv turbo-cdn /usr/local/bin/ && \
    chmod +x /usr/local/bin/turbo-cdn

# Copy download script
COPY scripts/download-deps.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/download-deps.sh

# Set cache directory
ENV TURBO_CDN_CACHE_DIR=/cache

# Run downloader
CMD ["download-deps.sh"]
```

### Download Script

```bash
#!/bin/bash
# scripts/download-deps.sh

set -e

echo "🚀 Turbo CDN Docker Downloader"
echo "=============================="

# Ensure cache directory exists
mkdir -p $TURBO_CDN_CACHE_DIR

# Define dependencies
DEPENDENCIES=(
    "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz"
    "https://golang.org/dl/go1.21.5.linux-amd64.tar.gz"
    "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz"
    "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-unknown-linux-gnu.tar.gz"
    "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-unknown-linux-gnu.tar.gz"
)

# Download dependencies
echo "📥 Downloading ${#DEPENDENCIES[@]} dependencies..."

for i in "${!DEPENDENCIES[@]}"; do
    url="${DEPENDENCIES[$i]}"
    filename=$(basename "$url")
    
    echo "[$((i+1))/${#DEPENDENCIES[@]}] Downloading: $filename"
    
    if [ ! -f "$TURBO_CDN_CACHE_DIR/$filename" ]; then
        turbo-cdn dl "$url" "$TURBO_CDN_CACHE_DIR/" --verbose
        echo "✅ Downloaded: $filename"
    else
        echo "⏭️  Cached: $filename"
    fi
done

echo "🎉 All dependencies ready in $TURBO_CDN_CACHE_DIR"

# Extract dependencies for immediate use
echo "📦 Extracting dependencies..."
cd $TURBO_CDN_CACHE_DIR

for archive in *.tar.gz *.tar.xz; do
    if [ -f "$archive" ]; then
        echo "📦 Extracting: $archive"
        tar -xf "$archive"
    fi
done

echo "✅ Dependencies extracted and ready"

# Keep container running for development
tail -f /dev/null
```

## 🏭 Production Docker Setup

### Production Dockerfile

```dockerfile
# Production-optimized Dockerfile with Turbo CDN
FROM ubuntu:22.04 AS base

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Turbo CDN
RUN curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz && \
    mv turbo-cdn /usr/local/bin/ && \
    chmod +x /usr/local/bin/turbo-cdn

# Download stage
FROM base AS downloader
WORKDIR /downloads

# Download production dependencies
RUN turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz" && \
    tar -xf node-v20.10.0-linux-x64.tar.xz

# Build stage
FROM base AS builder

# Copy Node.js from downloader
COPY --from=downloader /downloads/node-v20.10.0-linux-x64 /opt/node
ENV PATH="/opt/node/bin:$PATH"

# Copy source and build
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

# Production stage
FROM ubuntu:22.04 AS production

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r appuser && useradd -r -g appuser appuser

# Copy Node.js runtime
COPY --from=builder /opt/node /opt/node
ENV PATH="/opt/node/bin:$PATH"

# Copy application
WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

EXPOSE 3000
CMD ["node", "dist/index.js"]
```

## 🔧 Docker Build Optimization

### Build Script with Caching

```bash
#!/bin/bash
# scripts/docker-build.sh

set -e

echo "🐳 Docker Build with Turbo CDN Optimization"
echo "==========================================="

# Build arguments
BUILD_CONTEXT="."
IMAGE_NAME="myapp"
TAG="latest"
CACHE_DIR="./docker-cache"

# Create cache directory
mkdir -p $CACHE_DIR

# Build with cache mount
echo "🚀 Building Docker image with Turbo CDN..."

docker build \
    --tag $IMAGE_NAME:$TAG \
    --build-arg BUILDKIT_INLINE_CACHE=1 \
    --cache-from $IMAGE_NAME:cache \
    --target production \
    $BUILD_CONTEXT

echo "✅ Docker build completed!"

# Tag for cache
docker tag $IMAGE_NAME:$TAG $IMAGE_NAME:cache

# Show image size
echo "📊 Image size:"
docker images $IMAGE_NAME:$TAG --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"
```

### Docker Compose for Production

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
      target: production
    ports:
      - "80:3000"
    environment:
      - NODE_ENV=production
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    depends_on:
      - database
      - redis

  database:
    image: postgres:15-alpine
    environment:
      - POSTGRES_DB=myapp
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD_FILE=/run/secrets/db_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    secrets:
      - db_password
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    restart: unless-stopped
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data

  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - app
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:

secrets:
  db_password:
    file: ./secrets/db_password.txt
```

## 📊 Performance Comparison

### Build Time Comparison

```bash
# Traditional Docker build
time docker build -t myapp:traditional .
# Result: ~8-12 minutes

# With Turbo CDN optimization
time docker build -t myapp:turbo-cdn -f Dockerfile.turbo-cdn .
# Result: ~3-5 minutes (60% faster)
```

### Image Size Optimization

| Build Type | Image Size | Build Time | Download Speed |
|------------|------------|------------|----------------|
| Traditional | 1.2 GB | 10 min | 2-5 MB/s |
| Turbo CDN | 1.2 GB | 4 min | 8-15 MB/s |
| Multi-stage + Turbo CDN | 180 MB | 5 min | 8-15 MB/s |

## 🛠️ Development Workflow

### Development Docker Compose

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile.dev
    volumes:
      - .:/app
      - /app/node_modules
      - turbo-cdn-cache:/cache
    ports:
      - "3000:3000"
      - "9229:9229" # Debug port
    environment:
      - NODE_ENV=development
      - DEBUG=*
    command: npm run dev

  tools:
    build:
      context: .
      dockerfile: Dockerfile.tools
    volumes:
      - .:/workspace
      - turbo-cdn-cache:/cache
    working_dir: /workspace
    command: tail -f /dev/null

volumes:
  turbo-cdn-cache:
```

### Development Tools Dockerfile

```dockerfile
# Dockerfile.tools
FROM ubuntu:22.04

# Install Turbo CDN and development tools
RUN apt-get update && apt-get install -y curl git && \
    curl -L https://github.com/loonghao/turbo-cdn/releases/latest/download/turbo-cdn-x86_64-unknown-linux-gnu.tar.gz | tar xz && \
    mv turbo-cdn /usr/local/bin/

# Download development tools with Turbo CDN
WORKDIR /tools
RUN turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz" && \
    turbo-cdn dl "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-unknown-linux-gnu.tar.gz" && \
    turbo-cdn dl "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-unknown-linux-gnu.tar.gz"

# Extract tools
RUN for archive in *.tar.gz; do tar -xf "$archive"; done && \
    find . -name "rg" -o -name "fd" -o -name "bat" | xargs -I {} cp {} /usr/local/bin/

# Verify tools
RUN rg --version && fd --version && bat --version

WORKDIR /workspace
```

## 🔗 Next Steps

- [CI/CD Usage](ci_cd_usage.md) - Integrate with CI/CD pipelines
- [Performance Examples](../performance/) - Optimize download performance
- [API Examples](../api/) - Use Turbo CDN in your applications
