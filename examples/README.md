# 🚀 Turbo CDN Examples

This directory contains comprehensive examples demonstrating how to use Turbo CDN's CLI and API features.

## 📁 Directory Structure

```
examples/
├── README.md                    # This file
├── cli/                        # Command Line Interface examples
│   ├── basic_download.md       # Basic download commands
│   ├── advanced_usage.md       # Advanced CLI features
│   └── batch_operations.md     # Batch download examples
├── api/                        # Library API examples
│   ├── basic_usage.rs          # Basic API usage
│   ├── async_api.rs           # Async API examples
│   ├── advanced_config.rs     # Advanced configuration
│   └── integration.rs         # Integration examples
├── integration/                # Integration with other tools
│   ├── vx_integration.rs      # Integration with vx tool
│   ├── ci_cd_usage.md         # CI/CD pipeline examples
│   └── docker_usage.md        # Docker integration
└── performance/                # Performance optimization examples
    ├── benchmarks.rs          # Performance benchmarks
    ├── monitoring.rs          # Performance monitoring
    └── tuning.md              # Performance tuning guide
```

## 🎯 Quick Start

### CLI Examples
- [Basic Download](cli/basic_download.md) - Simple download commands
- [Advanced Usage](cli/advanced_usage.md) - Advanced CLI features
- [Batch Operations](cli/batch_operations.md) - Multiple file downloads

### API Examples
- [Basic Usage](api/basic_usage.rs) - Getting started with the API
- [Async API](api/async_api.rs) - Asynchronous operations
- [Advanced Config](api/advanced_config.rs) - Custom configurations

### Integration Examples
- [vx Integration](integration/vx_integration.rs) - Using with vx tool
- [CI/CD Usage](integration/ci_cd_usage.md) - Continuous integration
- [Docker Usage](integration/docker_usage.md) - Container environments

## 🏃‍♂️ Running Examples

### CLI Examples
```bash
# Follow the commands in the markdown files
cat examples/cli/basic_download.md
```

### API Examples
```bash
# Run Rust examples
cargo run --example basic_usage
cargo run --example async_api
cargo run --example advanced_config
```

### Performance Examples
```bash
# Run benchmarks
cargo run --example benchmarks
cargo run --example monitoring
```

## 📚 Documentation

For more detailed documentation, see:
- [Main README](../README.md)
- [vx Integration Guide](../docs/VX_INTEGRATION.md)
- [API Documentation](https://docs.rs/turbo-cdn)

## 🤝 Contributing

Found an issue or want to add more examples? Please contribute!

1. Fork the repository
2. Create your example branch (`git checkout -b examples/my-example`)
3. Add your example
4. Commit your changes (`git commit -am 'feat: add my example'`)
5. Push to the branch (`git push origin examples/my-example`)
6. Create a Pull Request
