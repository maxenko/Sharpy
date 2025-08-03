use image::{RgbImage, Rgb};
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum EdgeMethod {
    Sobel,
    Prewitt,
}

/// Applies Gaussian blur to an image with the specified radius.
/// 
/// Uses separable convolution for better performance on larger kernels.
pub fn gaussian_blur(img: &RgbImage, radius: f32) -> RgbImage {
    let (width, height) = img.dimensions();
    
    let kernel_size = (radius * 6.0).ceil() as usize | 1;
    let kernel = Arc::new(generate_gaussian_kernel(kernel_size, radius));
    let half_kernel = kernel_size / 2;
    
    // First pass: horizontal blur
    let mut temp = RgbImage::new(width, height);
    
    // Process in parallel chunks for better cache locality
    temp.enumerate_rows_mut()
        .par_bridge()
        .for_each(|(y, row)| {
            for (x, _, pixel) in row {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                let mut weight_sum = 0.0;
                
                for k in 0..kernel_size {
                    let img_x = (x as i32 + k as i32 - half_kernel as i32)
                        .max(0)
                        .min(width as i32 - 1) as u32;
                    
                    let source_pixel = img.get_pixel(img_x, y);
                    let weight = kernel[k];
                    
                    r_sum += source_pixel[0] as f32 * weight;
                    g_sum += source_pixel[1] as f32 * weight;
                    b_sum += source_pixel[2] as f32 * weight;
                    weight_sum += weight;
                }
                
                *pixel = Rgb([
                    (r_sum / weight_sum).round().clamp(0.0, 255.0) as u8,
                    (g_sum / weight_sum).round().clamp(0.0, 255.0) as u8,
                    (b_sum / weight_sum).round().clamp(0.0, 255.0) as u8,
                ]);
            }
        });
    
    // Second pass: vertical blur
    let mut result = RgbImage::new(width, height);
    let temp = Arc::new(temp);
    let kernel = Arc::clone(&kernel);
    
    // Process all pixels and collect results
    let pixel_values: Vec<_> = (0..width).into_par_iter()
        .flat_map(|x| {
            let temp = Arc::clone(&temp);
            let kernel = Arc::clone(&kernel);
            (0..height).into_par_iter().map(move |y| {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                let mut weight_sum = 0.0;
                
                for k in 0..kernel_size {
                    let img_y = (y as i32 + k as i32 - half_kernel as i32)
                        .max(0)
                        .min(height as i32 - 1) as u32;
                    
                    let source_pixel = temp.get_pixel(x, img_y);
                    let weight = kernel[k];
                    
                    r_sum += source_pixel[0] as f32 * weight;
                    g_sum += source_pixel[1] as f32 * weight;
                    b_sum += source_pixel[2] as f32 * weight;
                    weight_sum += weight;
                }
                
                let pixel = Rgb([
                    (r_sum / weight_sum).round().clamp(0.0, 255.0) as u8,
                    (g_sum / weight_sum).round().clamp(0.0, 255.0) as u8,
                    (b_sum / weight_sum).round().clamp(0.0, 255.0) as u8,
                ]);
                
                (x, y, pixel)
            })
        })
        .collect();
    
    // Apply all pixel values
    for (x, y, pixel) in pixel_values {
        result.put_pixel(x, y, pixel);
    }
    
    result
}

fn generate_gaussian_kernel(size: usize, sigma: f32) -> Vec<f32> {
    let mut kernel = vec![0.0; size];
    let half_size = size / 2;
    let two_sigma_sq = 2.0 * sigma * sigma;
    
    // 1D Gaussian kernel
    for i in 0..size {
        let x = i as f32 - half_size as f32;
        kernel[i] = (-x * x / two_sigma_sq).exp();
    }
    
    let sum: f32 = kernel.iter().sum();
    for value in &mut kernel {
        *value /= sum;
    }
    
    kernel
}

/// Applies a convolution kernel to an image.
/// 
/// Optimized for small kernels (3x3, 5x5) commonly used in sharpening.
pub fn apply_convolution(
    img: &RgbImage,
    kernel: &[f32],
    kernel_size: usize,
) -> RgbImage {
    let (width, height) = img.dimensions();
    let mut result = RgbImage::new(width, height);
    let half_kernel = kernel_size / 2;
    
    // Calculate all convolved pixels in parallel
    let pixel_values: Vec<_> = (0..height).into_par_iter()
        .flat_map(|y| {
            (0..width).into_par_iter().map(move |x| {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                
                for ky in 0..kernel_size {
                    for kx in 0..kernel_size {
                        let img_x = (x as i32 + kx as i32 - half_kernel as i32)
                            .max(0)
                            .min(width as i32 - 1) as u32;
                        let img_y = (y as i32 + ky as i32 - half_kernel as i32)
                            .max(0)
                            .min(height as i32 - 1) as u32;
                        
                        let source_pixel = img.get_pixel(img_x, img_y);
                        let weight = kernel[ky * kernel_size + kx];
                        
                        r_sum += source_pixel[0] as f32 * weight;
                        g_sum += source_pixel[1] as f32 * weight;
                        b_sum += source_pixel[2] as f32 * weight;
                    }
                }
                
                let pixel = Rgb([
                    r_sum.round().clamp(0.0, 255.0) as u8,
                    g_sum.round().clamp(0.0, 255.0) as u8,
                    b_sum.round().clamp(0.0, 255.0) as u8,
                ]);
                
                (x, y, pixel)
            })
        })
        .collect();
    
    // Apply all pixel values
    for (x, y, pixel) in pixel_values {
        result.put_pixel(x, y, pixel);
    }
    
    result
}

pub fn get_high_pass_kernel() -> ([f32; 9], usize) {
    (
        [
            0.0, -1.0, 0.0,
            -1.0, 5.0, -1.0,
            0.0, -1.0, 0.0,
        ],
        3,
    )
}

pub fn get_sobel_kernels() -> (([f32; 9], usize), ([f32; 9], usize)) {
    let x_kernel = (
        [
            -1.0, 0.0, 1.0,
            -2.0, 0.0, 2.0,
            -1.0, 0.0, 1.0,
        ],
        3,
    );
    
    let y_kernel = (
        [
            -1.0, -2.0, -1.0,
            0.0, 0.0, 0.0,
            1.0, 2.0, 1.0,
        ],
        3,
    );
    
    (x_kernel, y_kernel)
}

pub fn get_prewitt_kernels() -> (([f32; 9], usize), ([f32; 9], usize)) {
    let x_kernel = (
        [
            -1.0, 0.0, 1.0,
            -1.0, 0.0, 1.0,
            -1.0, 0.0, 1.0,
        ],
        3,
    );
    
    let y_kernel = (
        [
            -1.0, -1.0, -1.0,
            0.0, 0.0, 0.0,
            1.0, 1.0, 1.0,
        ],
        3,
    );
    
    (x_kernel, y_kernel)
}

/// Blends two images with the specified strength.
/// 
/// Optimized version that operates directly on pixel data.
pub fn blend_images(original: &RgbImage, processed: &RgbImage, strength: f32) -> RgbImage {
    let (width, height) = original.dimensions();
    let mut result = RgbImage::new(width, height);
    
    let blend_factor = strength.clamp(0.0, 1.0);
    let inv_blend = 1.0 - blend_factor;
    
    // Process all pixels in parallel
    let pixel_values: Vec<_> = (0..height).into_par_iter()
        .flat_map(|y| {
            (0..width).into_par_iter().map(move |x| {
                let orig_pixel = original.get_pixel(x, y);
                let proc_pixel = processed.get_pixel(x, y);
                
                let pixel = Rgb([
                    (orig_pixel[0] as f32 * inv_blend + proc_pixel[0] as f32 * blend_factor)
                        .round()
                        .clamp(0.0, 255.0) as u8,
                    (orig_pixel[1] as f32 * inv_blend + proc_pixel[1] as f32 * blend_factor)
                        .round()
                        .clamp(0.0, 255.0) as u8,
                    (orig_pixel[2] as f32 * inv_blend + proc_pixel[2] as f32 * blend_factor)
                        .round()
                        .clamp(0.0, 255.0) as u8,
                ]);
                
                (x, y, pixel)
            })
        })
        .collect();
    
    // Apply all pixel values
    for (x, y, pixel) in pixel_values {
        result.put_pixel(x, y, pixel);
    }
    
    result
}

pub fn calculate_luminance(pixel: &Rgb<u8>) -> f32 {
    0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32
}

/// Applies edge detection using the specified method.
/// 
/// Combines horizontal and vertical edge detection kernels.
pub fn apply_edge_detection(
    img: &RgbImage,
    method: EdgeMethod,
) -> RgbImage {
    let (x_kernel, y_kernel) = match method {
        EdgeMethod::Sobel => get_sobel_kernels(),
        EdgeMethod::Prewitt => get_prewitt_kernels(),
    };
    
    let x_edges = Arc::new(apply_convolution(img, &x_kernel.0, x_kernel.1));
    let y_edges = Arc::new(apply_convolution(img, &y_kernel.0, y_kernel.1));
    
    let (width, height) = img.dimensions();
    let mut result = RgbImage::new(width, height);
    
    // Calculate edge magnitudes in parallel
    let pixel_values: Vec<_> = (0..height).into_par_iter()
        .flat_map(|y| {
            let x_edges = Arc::clone(&x_edges);
            let y_edges = Arc::clone(&y_edges);
            (0..width).into_par_iter().map(move |x| {
                let x_pixel = x_edges.get_pixel(x, y);
                let y_pixel = y_edges.get_pixel(x, y);
                
                let x_mag = calculate_luminance(x_pixel);
                let y_mag = calculate_luminance(y_pixel);
                let magnitude = (x_mag * x_mag + y_mag * y_mag).sqrt().clamp(0.0, 255.0) as u8;
                
                let pixel = Rgb([magnitude, magnitude, magnitude]);
                (x, y, pixel)
            })
        })
        .collect();
    
    // Apply all pixel values
    for (x, y, pixel) in pixel_values {
        result.put_pixel(x, y, pixel);
    }
    
    result
}