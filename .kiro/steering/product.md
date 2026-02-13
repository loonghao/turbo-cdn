# Turbo CDN Product Overview

Turbo CDN is an intelligent download accelerator with automatic CDN optimization and concurrent chunked downloads. It's a Rust-based CLI tool and library that provides next-generation download acceleration through geographic detection, real-time CDN quality assessment, and comprehensive mirror optimization.

## Core Features

- **Intelligent CDN Optimization**: Automatic selection from 16+ CDN rules covering GitHub, jsDelivr, PyPI, Crates.io, Docker Hub, Maven, and more
- **Geographic Awareness**: Auto-detection of user location with region-specific CDN selection (China, Asia-Pacific, Europe, North America, Global)
- **High-Performance Architecture**: Concurrent chunked downloads with adaptive concurrency control and smart chunking algorithms
- **Real-time Quality Assessment**: Dynamic CDN performance monitoring with latency, bandwidth, and availability scoring
- **Resume Support**: Robust resume capability for interrupted downloads
- **Smart Download Mode**: Automatic method selection testing multiple approaches to find the fastest

## Target Use Cases

- Accelerating open source software downloads (GitHub releases, package managers)
- Integration with development tools and CI/CD pipelines
- Library integration for applications needing optimized downloads
- Command-line tool for developers and system administrators

## Architecture Philosophy

The project follows a modular, high-performance architecture with intelligent optimization modules:
- Adaptive systems that learn and optimize based on network conditions
- Geographic and performance-aware CDN selection
- Memory-optimized with mimalloc and high-performance HTTP clients
- Type-safe configuration system with comprehensive error handling