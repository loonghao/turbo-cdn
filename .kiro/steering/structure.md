# Project Structure & Organization

## Directory Layout

```
turbo-cdn/
├── src/                        # Source code
│   ├── lib.rs                  # Main library API and exports
│   ├── main.rs                 # CLI application entry point
│   ├── config/                 # Configuration system
│   │   ├── mod.rs              # Config types and loading
│   │   └── default.toml        # Default configuration
│   ├── error.rs                # Error types and handling
│   ├── logging.rs              # Logging configuration
│   ├── progress.rs             # Progress tracking utilities
│   ├── url_mapper.rs           # URL mapping and CDN rules
│   ├── concurrent_downloader.rs # Multi-threaded download engine
│   ├── smart_downloader.rs     # Intelligent download selection
│   ├── http_client.rs          # HTTP client wrapper
│   ├── http_client_manager.rs  # HTTP client pool management
│   ├── adaptive_concurrency.rs # Dynamic concurrency control
│   ├── adaptive_speed_controller.rs # Speed-based optimization
│   ├── smart_chunking.rs       # Intelligent file chunking
│   ├── dns_cache.rs            # DNS resolution caching
│   ├── load_balancer.rs        # Server selection and balancing
│   ├── cdn_quality.rs          # CDN performance assessment
│   ├── geo_detection.rs        # Geographic location detection
│   ├── server_quality_scorer.rs # Server performance scoring
│   ├── server_tracker.rs       # Server health monitoring
│   ├── cli_progress.rs         # CLI progress display
│   └── mmap_writer.rs          # Memory-mapped file writing
├── examples/                   # Usage examples
│   ├── api/                    # Library API examples
│   ├── cli/                    # CLI usage examples
│   ├── integration/            # Integration examples
│   └── performance/            # Performance benchmarks
├── tests/                      # Test suite
├── docs/                       # Documentation
├── scripts/                    # Build and maintenance scripts
├── benches/                    # Performance benchmarks
└── ci/                         # CI/CD configuration
```

## Module Architecture

### Core Modules
- **lib.rs**: Main API surface, re-exports, and high-level client
- **main.rs**: CLI application with clap-based argument parsing
- **config/**: Type-safe configuration system using figment
- **error.rs**: Centralized error handling with thiserror

### Network & Download Modules
- **concurrent_downloader.rs**: Core multi-threaded download engine
- **smart_downloader.rs**: Intelligent method selection and optimization
- **http_client.rs**: HTTP client abstraction and configuration
- **http_client_manager.rs**: Connection pooling and client lifecycle

### Performance Optimization Modules
- **adaptive_concurrency.rs**: Dynamic concurrency based on network conditions
- **adaptive_speed_controller.rs**: Speed-based parameter adjustment
- **smart_chunking.rs**: Intelligent file segmentation algorithms
- **dns_cache.rs**: High-performance DNS resolution caching
- **load_balancer.rs**: Server selection and health-based routing

### CDN & Geographic Modules
- **url_mapper.rs**: URL pattern matching and CDN rule engine
- **cdn_quality.rs**: Real-time CDN performance assessment
- **geo_detection.rs**: IP-based geographic location detection
- **server_quality_scorer.rs**: Server performance scoring algorithms
- **server_tracker.rs**: Server health monitoring and statistics

### Utility Modules
- **progress.rs**: Progress tracking and reporting abstractions
- **cli_progress.rs**: CLI-specific progress display
- **logging.rs**: Structured logging configuration
- **mmap_writer.rs**: Memory-mapped file I/O for performance

## Code Organization Patterns

### API Design
- **Sync & Async APIs**: Both blocking and async interfaces available
- **Builder Pattern**: Configuration and options use builder pattern
- **Result Types**: Consistent error handling with custom Result type
- **Progress Callbacks**: Flexible progress reporting system

### Configuration
- **Layered Config**: TOML files + environment variables + defaults
- **Type Safety**: Strong typing with serde deserialization
- **Feature Flags**: Optional features controlled via Cargo features
- **Runtime Config**: Some settings adjustable at runtime

### Testing Structure
- **Unit Tests**: Inline tests in each module
- **Integration Tests**: Full workflow tests in `tests/`
- **Property Tests**: Property-based testing with proptest
- **Benchmarks**: Performance tests in `benches/`

### Documentation
- **API Docs**: Comprehensive rustdoc documentation
- **Examples**: Working examples for all major features
- **Guides**: Step-by-step integration guides
- **Performance**: Benchmarking and optimization guides