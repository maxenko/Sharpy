# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-08-03

### Added
- Initial release of sharpy image sharpening library
- Four sharpening algorithms:
  - Unsharp mask - Classic sharpening method
  - High-pass sharpening - Convolution-based enhancement
  - Edge enhancement - Using Sobel and Prewitt operators
  - Clarity - Local contrast enhancement
- Builder pattern API for chaining operations
- Six built-in presets:
  - Subtle - Light sharpening for general use
  - Moderate - Balanced sharpening with clarity
  - Strong - Heavy sharpening for soft images
  - Edge-aware - Emphasizes edges while preserving smooth areas
  - Portrait - Optimized for portraits (avoids over-sharpening skin)
  - Landscape - Maximum detail extraction for landscapes
- CLI tool with commands:
  - `sharpy unsharp` - Apply unsharp mask
  - `sharpy highpass` - Apply high-pass sharpening
  - `sharpy edges` - Enhance edges
  - `sharpy clarity` - Apply clarity enhancement
  - `sharpy preset` - Use built-in presets
  - `sharpy batch` - Process multiple files
- Parallel processing using Rayon for optimal performance
- Copy-on-write semantics for efficient memory usage
- Comprehensive documentation and examples
- Benchmarks using Criterion

### Performance
- Separable convolution for Gaussian blur operations
- Chunk-based parallel processing for better cache locality
- Zero-copy operations where possible

### Dependencies
- image 0.25
- rayon 1.10
- thiserror 2.0
- clap 4.5 (CLI only)
- indicatif 0.18 (CLI only)
- glob 0.3 (CLI only)
- anyhow 1.0 (CLI only)

[Unreleased]: https://github.com/maxenko/sharpy/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/maxenko/sharpy/releases/tag/v0.1.0