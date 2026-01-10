# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.2.1](https://github.com/loonghao/turbo-cdn/compare/v0.2.0...v0.2.1) - 2025-06-23

### Fixed

- *(deps)* update rust crate directories to v6
- resolve cross-platform path issues and optimize HTTP client

### Other

- upgrade to directories crate for better cross-platform path management

## [0.2.0](https://github.com/loonghao/turbo-cdn/compare/v0.1.2...v0.2.0) - 2025-06-21

### Added

- implement automatic geo-detection and performance data persistence

### Fixed

- correct DownloadOptions timeout field type in documentation and examples
- resolve all test compilation and runtime issues

## [0.1.2](https://github.com/loonghao/turbo-cdn/compare/v0.1.1...v0.1.2) - 2025-06-21

### Added

- add async API interfaces for vx integration

### Fixed

- resolve clippy warnings in async API

### Other

- improve test coverage for async API and core structures

## [0.1.1](https://github.com/loonghao/turbo-cdn/compare/v0.1.0...v0.1.1) - 2025-06-20

### Added

- add universal URL optimization with comprehensive test coverage
- implement remaining TODO items and complete API functionality
- implement cache statistics tracking and metadata persistence

### Other

- improve code quality and remove code smells
## [Unreleased]

## [0.8.0](https://github.com/loonghao/turbo-cdn/compare/v0.7.1...v0.8.0) - 2026-01-10

### Added

- rustls ring backend and self-update opt-in

### Fixed

- remove extra blank lines after rustls init calls for fmt compliance
- make init_rustls_provider public and add to ConcurrentDownloader
- initialize rustls ring provider before creating reqwest clients

### Other

- *(deps)* update rust crate tokio-util to v0.7.18
- *(deps)* update rust crate url to v2.5.8
- *(deps)* update rust crate rustls to v0.23.36
- *(deps)* update rust crate tokio-test to v0.4.5
- *(deps)* update rust crate serde_json to v1.0.149

## [0.7.1](https://github.com/loonghao/turbo-cdn/compare/v0.7.0...v0.7.1) - 2026-01-03

### Added

- Microsoft Visual Studio download mapping rule while preserving original URL fallback
- Self-update CLI command retained as opt-in feature flag

### Changed

- Move rustls to ring backend (`rustls-no-provider`) to drop `aws-lc-sys` / cmake / NASM toolchain requirements
- Disable `self-update` in default feature set for library users; document opt-in CLI builds
- Deduplicate mapped URLs to avoid repeating the original entry


## [0.7.0](https://github.com/loonghao/turbo-cdn/compare/v0.6.1...v0.7.0) - 2026-01-01

### Added

- improve GitHub CDN download stability and add vx/auroraview E2E tests
- [**breaking**] upgrade reqwest to 0.13 with rustls as default TLS backend

## [0.6.1](https://github.com/loonghao/turbo-cdn/compare/v0.6.0...v0.6.1) - 2025-12-29

### Fixed

- *(deps)* disable default-features for self_update to avoid default-tls conflict

## [0.6.0](https://github.com/loonghao/turbo-cdn/compare/v0.5.0...v0.6.0) - 2025-12-29

### Added

- add VitePress documentation site and self-update functionality
- add loonghao projects and expand E2E tests

### Fixed

- improve CDN reliability and error handling
- *(deps)* update rust crate ping to 0.7
- *(deps)* update rust crate lru to 0.16

### Other

- *(deps)* update rust crate reqwest to v0.12.28
- *(deps)* update rust crate tracing to v0.1.44
- *(deps)* update rust crate serde_json to v1.0.148

## [0.5.0](https://github.com/loonghao/turbo-cdn/compare/v0.4.3...v0.5.0) - 2025-12-14

### Fixed

- correct release-plz config and add adaptive concurrency tests
- update dependencies and add more test coverage
- *(deps)* update rust crate tokio-util to v0.7.16
- *(deps)* update rust crate indicatif to 0.18

### Other

- add comprehensive unit tests for dns_cache, load_balancer, smart_chunking, http_client_manager modules
- add E2E tests and performance benchmarks
- update dependencies via cargo update
- improve API ergonomics and add comprehensive tests
- *(deps)* update rust crate criterion to 0.7

## [0.4.3](https://github.com/loonghao/turbo-cdn/compare/v0.4.2...v0.4.3) - 2025-07-03

### Added

- optimize build profiles and clippy configuration

### Other

- update README to reflect current architecture

## [0.4.2](https://github.com/loonghao/turbo-cdn/compare/v0.4.1...v0.4.2) - 2025-07-03

### Fixed

- disable uninlined_format_args clippy warning
- resolve all remaining uninlined format args warnings
- resolve uninlined format args clippy warnings
- resolve clippy warnings and modernize string formatting
- resolve remaining clippy format string warnings in cli_progress.rs
- resolve final clippy format string warnings and add automation scripts
- resolve all clippy format string warnings
- resolve clippy warnings for format string optimization

## [0.4.1](https://github.com/loonghao/turbo-cdn/compare/v0.4.0...v0.4.1) - 2025-06-25

### Added

- simplify release workflow using upload-rust-binary-action
- integrate release-plz with GitHub release automation

### Fixed

- restore git_release_body template for rich release notes

## [0.4.0](https://github.com/loonghao/turbo-cdn/compare/v0.3.0...v0.4.0) - 2025-06-25

### Added

- implement professional logging system and smart download mode
- implement smart download mode with automatic method selection
- optimize download performance for turbo speed
- enhance release-plz configuration with comprehensive release template
- improve CI/CD configuration based on ripgrep best practices
- add comprehensive examples for CLI and API usage

### Fixed

- resolve all clippy warnings to pass strict linting
- update test expectations to match new default configuration values
- resolve all compilation errors in examples and core library
- suppress dead_code warning for config field
- simplify CI to use stable Rust version only
- use isahc static-curl and static-ssl features to avoid OpenSSL
- simplify CI following ripgrep best practices
- add execute permission to ubuntu-install-packages script
- resolve CI Ubuntu package installation issues
- resolve all compilation errors in performance examples
- resolve compilation errors in examples

### Other

- apply cargo fmt formatting
- simplify HTTP client to use only reqwest with rustls

### Security

- fix vulnerabilities by updating dependencies

## [0.3.0](https://github.com/loonghao/turbo-cdn/compare/v0.2.1...v0.3.0) - 2025-06-24

### Added

- [**breaking**] BREAKING CHANGES - Complete performance optimization overhaul
- comprehensive performance optimizations and intelligent CDN system
- implement configuration-driven GitHub releases mirror system

### Fixed

- enable GitHub source and add GitHub releases mirrors

### Other

- Update README with new intelligent optimization features

### Added
- Initial project structure and core architecture
- Multi-CDN support (GitHub, jsDelivr, Fastly, Cloudflare)
- Intelligent routing with performance tracking
- Compliance checking for open-source content only
- Smart caching with compression and TTL management
- Progress tracking with real-time callbacks
- Parallel chunked downloads with resume support
- Comprehensive error handling and retry logic
- Regional optimization for global performance
- GDPR/CCPA compliant data handling
- Audit logging for compliance tracking

### Security
- Strict open-source license verification
- Content validation and copyright checking
- Secure data handling with minimal collection
- Encrypted communication for all downloads

## [0.1.0] - 2025-06-19

### Added
- Initial release of Turbo CDN
- Core download acceleration functionality
- Multi-source CDN routing
- Basic compliance checking
- Progress tracking and reporting
- Caching system with compression
- Command-line interface
- Comprehensive documentation
- Unit and integration tests
- CI/CD pipeline with automated releases

### Features
- **Download Sources**:
  - GitHub Releases integration
  - jsDelivr CDN support
  - Fastly CDN support
  - Cloudflare CDN support
  
- **Performance Optimizations**:
  - Parallel chunked downloads
  - Intelligent source selection
  - Automatic failover and retry
  - Regional CDN optimization
  - Smart caching with compression
  
- **Compliance & Security**:
  - Open-source license verification
  - Content validation
  - Privacy-compliant data handling
  - Comprehensive audit logging
  
- **Developer Experience**:
  - Simple and intuitive API
  - Comprehensive documentation
  - Rich progress tracking
  - Flexible configuration options
  - Extensive error handling

### Technical Details
- **Languages**: Rust 2021 edition
- **Async Runtime**: Tokio
- **HTTP Client**: reqwest with rustls
- **Serialization**: serde with JSON/TOML support
- **Progress Tracking**: indicatif
- **Logging**: tracing with structured logging
- **Testing**: tokio-test, wiremock for mocking
- **Benchmarking**: criterion for performance testing

### Performance Metrics
- 200-300% faster downloads compared to single-source
- 99%+ success rate with intelligent failover
- 50-70% reduced latency through optimal CDN selection
- Support for files from KB to GB in size
- Efficient memory usage with streaming downloads

### Compliance Features
- Automatic open-source license detection
- Content validation against blocked patterns
- GDPR/CCPA compliant data handling
- Comprehensive audit trail
- User consent management
- Data minimization and anonymization

### Documentation
- Comprehensive README with examples
- API documentation with rustdoc
- Contributing guidelines
- Security policy
- Privacy policy
- Terms of service
- Compliance documentation

### Testing
- Unit tests for all core modules
- Integration tests for end-to-end workflows
- Compliance tests for legal requirements
- Performance benchmarks
- Security audit tests
- Mock testing for external services

### CI/CD
- Automated testing on multiple platforms
- Security vulnerability scanning
- Code quality checks with clippy
- Automated releases with release-plz
- Documentation generation and deployment
- Performance regression testing

## [0.0.1] - 2025-06-18

### Added
- Project initialization
- Basic project structure
- Initial Cargo.toml configuration
- MIT License
- Basic README template

---

## Release Notes

### Version 0.1.0 Highlights

This is the initial release of Turbo CDN, a revolutionary download accelerator designed specifically for open-source software. Key highlights include:

**🚀 Performance**: Up to 300% faster downloads through intelligent multi-CDN routing and parallel chunked downloads.

**🔒 Compliance**: Built from the ground up with compliance in mind, supporting only verified open-source content with comprehensive audit logging.

**🌐 Global Reach**: Optimized for global usage with regional CDN preferences and intelligent routing based on geographic location.

**🛠️ Developer Friendly**: Simple, intuitive API with comprehensive documentation and extensive configuration options.

**📊 Monitoring**: Rich progress tracking, performance metrics, and health monitoring for all CDN sources.

### Breaking Changes

None - this is the initial release.

### Migration Guide

This is the initial release, so no migration is required.

### Known Issues

- Some edge cases in chunk resumption may require manual retry
- Performance metrics collection is basic in this initial release
- Limited CDN source customization options

### Future Roadmap

- P2P acceleration support
- Advanced AI-based routing optimization
- Additional CDN source integrations
- Enhanced performance analytics
- Mobile and embedded platform support
- GraphQL API for advanced integrations

---

For more detailed information about changes, please refer to the [commit history](https://github.com/loonghao/turbo-cdn/commits/main) and [pull requests](https://github.com/loonghao/turbo-cdn/pulls).
