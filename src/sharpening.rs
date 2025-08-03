use crate::{Image, Result};
use crate::utils::{
    EdgeMethod, gaussian_blur, apply_convolution, get_high_pass_kernel,
    apply_edge_detection, blend_images, calculate_luminance,
};
use rayon::prelude::*;
use std::sync::Arc;

/// Applies unsharp masking to sharpen an image.
/// 
/// # Parameters
/// - `radius`: Blur radius for the mask (0.5-10.0)
/// - `amount`: Strength of sharpening (0.0-5.0)
/// - `threshold`: Minimum difference to apply sharpening (0-255)
pub fn unsharp_mask(mut image: Image, radius: f32, amount: f32, threshold: u8) -> Result<Image> {
    let original = Arc::new(image.data.get_ref().clone());
    let blurred = Arc::new(gaussian_blur(&original, radius));
    
    let buffer = image.data.get_mut();
    let (width, height) = buffer.dimensions();
    
    // Process pixels in parallel and collect results
    let pixel_values: Vec<_> = (0..height).into_par_iter()
        .flat_map(|y| {
            let original = Arc::clone(&original);
            let blurred = Arc::clone(&blurred);
            (0..width).into_par_iter().map(move |x| {
                let orig_pixel = original.get_pixel(x, y);
                let blur_pixel = blurred.get_pixel(x, y);
                
                let mut new_pixel = [0u8; 3];
                for i in 0..3 {
                    let orig_val = orig_pixel[i] as f32;
                    let blur_val = blur_pixel[i] as f32;
                    let diff = orig_val - blur_val;
                    
                    if diff.abs() > threshold as f32 {
                        let sharpened = orig_val + (diff * amount);
                        new_pixel[i] = sharpened.round().clamp(0.0, 255.0) as u8;
                    } else {
                        new_pixel[i] = orig_pixel[i];
                    }
                }
                
                (x, y, image::Rgb(new_pixel))
            })
        })
        .collect();
    
    // Apply all pixel values
    for (x, y, pixel) in pixel_values {
        buffer.put_pixel(x, y, pixel);
    }
    
    Ok(image)
}

/// Applies high-pass sharpening using a convolution kernel.
/// 
/// # Parameters
/// - `strength`: Blend strength with original image (0.0-3.0)
pub fn high_pass_sharpen(mut image: Image, strength: f32) -> Result<Image> {
    let original = image.data.get_ref().clone();
    let (kernel, kernel_size) = get_high_pass_kernel();
    let sharpened = apply_convolution(&original, &kernel, kernel_size);
    
    let buffer = image.data.get_mut();
    *buffer = blend_images(&original, &sharpened, strength);
    
    Ok(image)
}

/// Enhances edges in an image using edge detection.
/// 
/// # Parameters
/// - `strength`: Edge enhancement strength (0.0-3.0)
/// - `method`: Edge detection method (Sobel or Prewitt)
pub fn enhance_edges(mut image: Image, strength: f32, method: EdgeMethod) -> Result<Image> {
    let original = Arc::new(image.data.get_ref().clone());
    let edges = Arc::new(apply_edge_detection(&original, method));
    
    let buffer = image.data.get_mut();
    
    let (width, height) = buffer.dimensions();
    
    // Process pixels in parallel and collect results
    let pixel_values: Vec<_> = (0..height).into_par_iter()
        .flat_map(|y| {
            let original = Arc::clone(&original);
            let edges = Arc::clone(&edges);
            (0..width).into_par_iter().map(move |x| {
                let orig_pixel = original.get_pixel(x, y);
                let edge_pixel = edges.get_pixel(x, y);
                
                let edge_strength = calculate_luminance(edge_pixel) / 255.0;
                let enhancement = edge_strength * strength;
                
                let mut new_pixel = [0u8; 3];
                for i in 0..3 {
                    let orig_val = orig_pixel[i] as f32;
                    let enhanced = orig_val + (edge_strength * 255.0 * enhancement);
                    new_pixel[i] = enhanced.round().clamp(0.0, 255.0) as u8;
                }
                
                (x, y, image::Rgb(new_pixel))
            })
        })
        .collect();
    
    // Apply all pixel values
    for (x, y, pixel) in pixel_values {
        buffer.put_pixel(x, y, pixel);
    }
    
    Ok(image)
}

/// Applies clarity enhancement to improve local contrast.
/// 
/// # Parameters
/// - `strength`: Enhancement strength (0.0-3.0)
/// - `radius`: Local area radius (1.0-20.0)
pub fn clarity(mut image: Image, strength: f32, radius: f32) -> Result<Image> {
    let original = image.data.get_ref().clone();
    let (width, height) = original.dimensions();
    
    let buffer = image.data.get_mut();
    
    let window_size = (radius * 2.0).round() as usize;
    let half_window = window_size / 2;
    
    // Calculate enhancements first, then apply
    let original = Arc::new(original);
    let enhancements: Vec<_> = (0..height).into_par_iter()
        .flat_map(|y| {
            let original = Arc::clone(&original);
            (0..width).into_par_iter().map(move |x| {
                let orig_pixel = original.get_pixel(x, y);
                let orig_luminance = calculate_luminance(orig_pixel);
                
                let mut local_sum = 0.0;
                let mut count = 0;
                
                // Calculate local average luminance
                for dy in -(half_window as i32)..=(half_window as i32) {
                    for dx in -(half_window as i32)..=(half_window as i32) {
                        let nx = (x as i32 + dx).max(0).min(width as i32 - 1) as u32;
                        let ny = (y as i32 + dy).max(0).min(height as i32 - 1) as u32;
                        
                        let neighbor_pixel = original.get_pixel(nx, ny);
                        local_sum += calculate_luminance(neighbor_pixel);
                        count += 1;
                    }
                }
                
                let local_avg = local_sum / count as f32;
                let contrast_diff = orig_luminance - local_avg;
                
                // Apply stronger enhancement to midtones
                let midtone_factor = if orig_luminance > 64.0 && orig_luminance < 192.0 {
                    1.0
                } else {
                    0.5
                };
                
                let enhancement = contrast_diff * strength * midtone_factor * 0.5;
                (x, y, enhancement)
            })
        })
        .collect();
    
    // Apply enhancements to buffer
    for (x, y, enhancement) in enhancements {
        let orig_pixel = original.get_pixel(x, y);
        let pixel = buffer.get_pixel_mut(x, y);
        for i in 0..3 {
            let enhanced = orig_pixel[i] as f32 + enhancement;
            pixel[i] = enhanced.round().clamp(0.0, 255.0) as u8;
        }
    }
    
    Ok(image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbImage, Rgb};
    use crate::Image;
    
    fn create_test_image() -> Image {
        let mut img = RgbImage::new(100, 100);
        
        // Create a checkerboard pattern
        for y in 0..100 {
            for x in 0..100 {
                let value = if (x / 10 + y / 10) % 2 == 0 { 100 } else { 200 };
                img.put_pixel(x, y, Rgb([value, value, value]));
            }
        }
        
        Image::from_rgb(img)
    }
    
    #[test]
    fn test_unsharp_mask() {
        let img = create_test_image();
        let result = unsharp_mask(img, 1.0, 1.0, 0);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_high_pass_sharpen() {
        let img = create_test_image();
        let result = high_pass_sharpen(img, 0.5);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_enhance_edges() {
        let img = create_test_image();
        let result = enhance_edges(img, 1.0, EdgeMethod::Sobel);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_clarity() {
        let img = create_test_image();
        let result = clarity(img, 1.0, 2.0);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_chain_operations() {
        let img = create_test_image();
        let result = unsharp_mask(img, 0.5, 0.5, 0)
            .and_then(|img| high_pass_sharpen(img, 0.3))
            .and_then(|img| clarity(img, 0.5, 1.0));
        assert!(result.is_ok());
    }
}