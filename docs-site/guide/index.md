# Introduction

Turbo CDN is a next-generation intelligent download accelerator built in Rust. It automatically optimizes download speeds through geographic detection, CDN quality assessment, and smart download strategies.

## Why Turbo CDN?

### The Problem

Downloading files from the internet can be slow and unreliable due to:

- **Geographic Distance**: Servers may be far from your location
- **Network Congestion**: Popular servers can become overloaded
- **Single Point of Failure**: Direct downloads have no fallback options
- **Manual Configuration**: Users must manually find and configure mirrors

### The Solution

Turbo CDN solves these problems with:

- **Automatic Geographic Detection**: Identifies your region and selects optimal mirrors
- **Real-time Quality Assessment**: Continuously monitors CDN performance
- **Smart Download Mode**: Automatically chooses the fastest download method
- **Comprehensive Mirror Coverage**: 16+ CDN rules across 6+ package managers

## Key Features

### 🌐 Intelligent Geographic Detection

- Automatic IP geolocation with multiple API fallbacks
- Network performance testing when IP detection fails
- Smart caching to avoid repeated detection calls
- Optimized for China, Asia-Pacific, Europe, North America, and Global regions

### 📊 Real-time CDN Quality Assessment

- Performance monitoring: latency, bandwidth, and availability
- Comprehensive 0-100 quality scoring algorithm
- Dynamic URL ranking based on real-time performance
- Background asynchronous quality evaluation

### ⚡ High-Performance Architecture

- **mimalloc**: High-performance memory allocator
- **reqwest + rustls**: Cross-platform HTTP client with TLS
- **Adaptive Concurrency**: Network condition-based parallelization
- **Smart Chunking**: IDM-style adaptive chunk sizing
- **DNS Caching**: High-performance resolution with hickory-dns

### 🔗 Extensive CDN Coverage

| Source | Mirrors | Regions |
|--------|---------|---------|
| GitHub | 7 mirrors | China, Asia, Global |
| PyPI | 3 mirrors | China |
| Crates.io | 2 mirrors | China |
| Go Modules | 2 mirrors | China |
| Docker Hub | 3 mirrors | China |
| Maven | 2 mirrors | China |
| jsDelivr | 5 nodes | Global |

## Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Input URL     │───▶│ Geographic       │───▶│ CDN Quality     │
│                 │    │ Detection        │    │ Assessment      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ URL Mapper      │    │ Real-time        │    │ Dynamic         │
│ (16+ Rules)     │    │ Performance      │    │ Ranking         │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Adaptive        │    │ Smart Chunking   │    │ DNS Cache       │
│ Concurrency     │    │ Algorithm        │    │ System          │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Load Balancer   │    │ High-Performance │    │ Downloaded      │
│ (Multi-Strategy)│    │ HTTP Client      │    │ File            │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Use Cases

### CLI Tool

Perfect for:
- Downloading GitHub releases
- Fetching package manager artifacts
- Batch downloading with CDN optimization

### Library Integration

Ideal for:
- Build tools (like [vx](https://github.com/loonghao/vx))
- Package managers
- CI/CD pipelines
- Any application needing fast, reliable downloads

## Next Steps

- [Quick Start](/guide/getting-started) - Get up and running in minutes
- [Installation](/guide/installation) - Detailed installation options
- [API Reference](/api/) - Complete API documentation
