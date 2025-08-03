# Sharpy Improvement Plan

## Overview
This document outlines a step-by-step plan to address identified issues in the sharpy crate while maintaining all core functionality. The improvements are organized by priority and grouped into logical work units.

## Priority 1: Critical Performance & Memory Issues

### 1.1 Optimize Parallel Processing (High Priority)
**Issue**: Current implementation collects all pixel values into vectors before processing, causing excessive memory usage.

**Steps**:
1. **Refactor pixel processing to use iterators instead of collecting**
   - Modify `src/algorithms/unsharp_mask.rs` to process pixels in chunks
   - Update parallel iteration to work directly on pixel slices
   - Implement streaming processing for large images
   
2. **Implement chunked processing**
   ```rust
   // Instead of: pixels.par_iter().collect() 
   // Use: pixels.par_chunks(chunk_size).map(process_chunk)
   ```

3. **Add memory-efficient convolution**
   - Process image in tiles to reduce memory footprint
   - Implement sliding window approach for kernel operations

**Testing**: Benchmark memory usage before/after with large images (8K+)

### 1.2 Fix Double Cloning in Arc Operations (High Priority)
**Issue**: Unnecessary cloning of image data when using Arc.

**Steps**:
1. **Audit all Arc usage in the codebase**
   - Search for `Arc::new(image.clone())` patterns
   - Replace with `Arc::clone(&arc_image)` where appropriate
   
2. **Implement copy-on-write properly**
   - Only clone when mutation is needed
   - Share immutable data through Arc without cloning

3. **Update builder pattern**
   - Store Arc<Image> internally
   - Clone only on final apply()

### 1.3 Add Memory Bounds Checking (Medium Priority)
**Issue**: No validation for maximum image size that can be safely processed.

**Steps**:
1. **Define memory limits**
   ```rust
   const MAX_IMAGE_PIXELS: usize = 100_000_000; // ~100MP
   const MAX_MEMORY_MB: usize = 4096; // 4GB default
   ```

2. **Add validation in Image::load()**
   - Check dimensions before allocating
   - Return error for images exceeding limits
   
3. **Implement progressive loading**
   - For large images, offer tiled processing option
   - Add `Image::load_progressive()` method

## Priority 2: Testing & Quality Assurance

### 2.1 Add Integration Tests (High Priority)
**Issue**: No tests validating actual image processing results.

**Steps**:
1. **Create test image dataset**
   - Add `tests/fixtures/` directory
   - Include various test images (small, medium, edge cases)
   
2. **Implement visual regression tests**
   ```rust
   #[test]
   fn test_unsharp_mask_visual() {
       let input = Image::load("tests/fixtures/test.jpg").unwrap();
       let output = input.unsharp_mask(1.0, 1.0, 0).unwrap();
       assert_images_similar(&output, &expected, 0.01);
   }
   ```

3. **Add algorithm correctness tests**
   - Test mathematical properties (e.g., convolution linearity)
   - Validate edge cases (black/white images, single pixel)

### 2.2 Add Memory Usage Tests (Medium Priority)
**Issue**: No validation of memory consumption.

**Steps**:
1. **Create memory benchmarks**
   ```rust
   #[bench]
   fn bench_memory_usage_4k_image() {
       let allocator = TrackingAllocator::new();
       // Process image and measure peak memory
   }
   ```

2. **Add stress tests**
   - Process multiple large images concurrently
   - Validate memory is properly released

### 2.3 Add Error Condition Tests (Medium Priority)
**Issue**: Missing tests for error paths.

**Steps**:
1. **Test all error conditions**
   - Invalid parameters
   - Corrupted image files
   - Out of memory scenarios
   
2. **Improve error messages**
   - Add context to errors
   - Provide actionable error messages

## Priority 3: API & Code Organization

### 3.1 Consolidate Duplicate Code (Medium Priority)
**Issue**: Operation enum defined in both CLI and library.

**Steps**:
1. **Move Operation enum to library**
   - Create `src/operations.rs`
   - Export from library root
   
2. **Update CLI to use library types**
   - Remove duplicate definitions
   - Import from library

3. **Create unified parameter validation**
   - Single source of truth for parameter ranges
   - Shared validation logic

### 3.2 Refactor CLI Validation (Medium Priority)
**Issue**: File checks happen after image loading.

**Steps**:
1. **Reorder CLI logic**
   ```rust
   // 1. Parse arguments
   // 2. Validate file paths and permissions
   // 3. Check overwrite conditions
   // 4. Load and process images
   ```

2. **Add pre-flight checks**
   - Verify output directory exists
   - Check write permissions
   - Validate input files exist

### 3.3 Improve Builder Validation (Low Priority)
**Issue**: No validation until apply() is called.

**Steps**:
1. **Add incremental validation**
   - Validate parameters as they're set
   - Return Result from builder methods
   
2. **Implement builder state machine**
   - Type-safe builder pattern
   - Compile-time validation where possible

## Priority 4: Security & Safety

### 4.1 Path Traversal Protection (Low Priority)
**Issue**: Potential path traversal in batch processing.

**Steps**:
1. **Sanitize glob patterns**
   - Validate paths stay within working directory
   - Reject absolute paths in batch mode
   
2. **Add path validation utility**
   ```rust
   fn validate_safe_path(path: &Path) -> Result<()> {
       // Check for .., absolute paths, symlinks
   }
   ```

### 4.2 Input Sanitization (Low Priority)
**Issue**: Limited input validation beyond clap.

**Steps**:
1. **Add parameter range validation**
   - Enforce reasonable limits
   - Prevent integer overflow
   
2. **Validate image data**
   - Check for malformed headers
   - Validate pixel data ranges

## Priority 5: Documentation & Code Quality

### 5.1 Improve Documentation (Low Priority)
**Steps**:
1. **Add missing function documentation**
   - Document all public APIs
   - Add examples for complex functions
   
2. **Create architecture document**
   - Explain design decisions
   - Document performance characteristics

### 5.2 Refactor Large Functions (Low Priority)
**Steps**:
1. **Break down complex functions**
   - Extract helper functions
   - Improve readability
   
2. **Add code comments**
   - Explain non-obvious algorithms
   - Document performance trade-offs

## Implementation Order

### Phase 1 (Week 1-2): Critical Performance
1. Optimize parallel processing (1.1)
2. Fix Arc cloning issues (1.2)
3. Add basic integration tests (2.1)

### Phase 2 (Week 3-4): Memory & Testing
1. Add memory bounds checking (1.3)
2. Add memory usage tests (2.2)
3. Add error condition tests (2.3)

### Phase 3 (Week 5): Code Organization
1. Consolidate duplicate code (3.1)
2. Refactor CLI validation (3.2)
3. Improve builder validation (3.3)

### Phase 4 (Week 6): Polish
1. Security improvements (4.1, 4.2)
2. Documentation updates (5.1, 5.2)
3. Final benchmarking and optimization

## Success Metrics

1. **Performance**
   - Memory usage reduced by 50% for large images
   - Processing time maintained or improved
   
2. **Quality**
   - Test coverage > 80%
   - All error paths tested
   
3. **Usability**
   - Clear error messages
   - Consistent API design
   
4. **Security**
   - No path traversal vulnerabilities
   - Safe handling of untrusted input

## Backward Compatibility

All changes will maintain backward compatibility:
- Public API signatures unchanged
- CLI commands work identically
- Output quality identical or improved

Breaking changes (if any) will be documented and versioned appropriately.