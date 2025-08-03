use image::{DynamicImage, RgbImage};
use std::sync::Arc;
use std::path::Path;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

mod sharpening;
mod utils;
mod builder;

pub use utils::EdgeMethod;
pub use builder::{SharpeningBuilder, SharpeningPresets};

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    
    #[error("Invalid parameter: {param} = {value}")]
    InvalidParameter { param: String, value: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Image format error: {0}")]
    Format(#[from] image::ImageError),
}

pub type Result<T> = std::result::Result<T, ImageError>;

#[derive(Clone)]
enum ImageData {
    Owned(RgbImage),
    Shared(Arc<RgbImage>),
}

impl ImageData {
    fn get_mut(&mut self) -> &mut RgbImage {
        match self {
            ImageData::Owned(img) => img,
            ImageData::Shared(arc_img) => {
                *self = ImageData::Owned((**arc_img).clone());
                if let ImageData::Owned(img) = self {
                    img
                } else {
                    unreachable!()
                }
            }
        }
    }
    
    fn get_ref(&self) -> &RgbImage {
        match self {
            ImageData::Owned(img) => img,
            ImageData::Shared(arc_img) => arc_img,
        }
    }
}

#[derive(Clone)]
pub struct Image {
    data: ImageData,
}

impl Image {
    pub fn from_dynamic(img: DynamicImage) -> Self {
        Self {
            data: ImageData::Owned(img.to_rgb8()),
        }
    }
    
    pub fn from_rgb(img: RgbImage) -> Self {
        Self {
            data: ImageData::Owned(img),
        }
    }
    
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let img = image::open(path)?;
        Ok(Self::from_dynamic(img))
    }
    
    pub fn from_arc_dynamic(arc_img: Arc<DynamicImage>) -> Self {
        match Arc::try_unwrap(arc_img) {
            Ok(img) => Self::from_dynamic(img),
            Err(arc_img) => Self {
                data: ImageData::Shared(Arc::new(arc_img.to_rgb8())),
            },
        }
    }
    
    pub fn from_arc_rgb(arc_img: Arc<RgbImage>) -> Self {
        match Arc::try_unwrap(arc_img) {
            Ok(img) => Self::from_rgb(img),
            Err(arc_img) => Self {
                data: ImageData::Shared(arc_img),
            },
        }
    }
    
    pub fn from_dynamic_ref(img: &DynamicImage) -> Self {
        Self {
            data: ImageData::Owned(img.to_rgb8()),
        }
    }
    
    pub fn into_arc_dynamic(self) -> Arc<DynamicImage> {
        match self.data {
            ImageData::Owned(img) => Arc::new(DynamicImage::ImageRgb8(img)),
            ImageData::Shared(arc_img) => {
                Arc::new(DynamicImage::ImageRgb8((*arc_img).clone()))
            }
        }
    }
    
    pub fn into_dynamic(self) -> DynamicImage {
        match self.data {
            ImageData::Owned(img) => DynamicImage::ImageRgb8(img),
            ImageData::Shared(arc_img) => {
                DynamicImage::ImageRgb8((*arc_img).clone())
            }
        }
    }
    
    pub fn into_rgb(self) -> RgbImage {
        match self.data {
            ImageData::Owned(img) => img,
            ImageData::Shared(arc_img) => (*arc_img).clone(),
        }
    }
    
    pub fn save<P: AsRef<Path>>(self, path: P) -> Result<()> {
        self.into_dynamic().save(path)?;
        Ok(())
    }
    
    pub fn dimensions(&self) -> (u32, u32) {
        self.data.get_ref().dimensions()
    }
    
    pub fn histogram(&self) -> [u32; 256] {
        let hist: Vec<AtomicU32> = (0..256).map(|_| AtomicU32::new(0)).collect();
        let img = self.data.get_ref();
        
        img.pixels().par_bridge().for_each(|pixel| {
            let [r, g, b] = pixel.0;
            let luminance = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as usize;
            hist[luminance.min(255)].fetch_add(1, Ordering::Relaxed);
        });
        
        let mut result = [0u32; 256];
        for (i, atomic_val) in hist.iter().enumerate() {
            result[i] = atomic_val.load(Ordering::Relaxed);
        }
        result
    }
    
    pub fn unsharp_mask(self, radius: f32, amount: f32, threshold: u8) -> Result<Self> {
        if radius <= 0.0 || radius > 10.0 {
            return Err(ImageError::InvalidParameter {
                param: "radius".to_string(),
                value: radius.to_string(),
            });
        }
        if amount < 0.0 || amount > 5.0 {
            return Err(ImageError::InvalidParameter {
                param: "amount".to_string(),
                value: amount.to_string(),
            });
        }
        
        sharpening::unsharp_mask(self, radius, amount, threshold)
    }
    
    pub fn high_pass_sharpen(self, strength: f32) -> Result<Self> {
        if strength <= 0.0 || strength > 3.0 {
            return Err(ImageError::InvalidParameter {
                param: "strength".to_string(),
                value: strength.to_string(),
            });
        }
        
        sharpening::high_pass_sharpen(self, strength)
    }
    
    pub fn enhance_edges(self, strength: f32, method: EdgeMethod) -> Result<Self> {
        if strength <= 0.0 || strength > 3.0 {
            return Err(ImageError::InvalidParameter {
                param: "strength".to_string(),
                value: strength.to_string(),
            });
        }
        
        sharpening::enhance_edges(self, strength, method)
    }
    
    pub fn clarity(self, strength: f32, radius: f32) -> Result<Self> {
        if strength <= 0.0 || strength > 3.0 {
            return Err(ImageError::InvalidParameter {
                param: "strength".to_string(),
                value: strength.to_string(),
            });
        }
        if radius <= 0.0 || radius > 20.0 {
            return Err(ImageError::InvalidParameter {
                param: "radius".to_string(),
                value: radius.to_string(),
            });
        }
        
        sharpening::clarity(self, strength, radius)
    }
    
    /// Creates a sharpening builder for fluent configuration.
    /// 
    /// # Example
    /// ```no_run
    /// # use sharpy::Image;
    /// # let image = Image::from_rgb(image::RgbImage::new(100, 100));
    /// let sharpened = image.sharpen()
    ///     .unsharp_mask(1.0, 1.0, 0)
    ///     .clarity(0.5, 2.0)
    ///     .apply()
    ///     .unwrap();
    /// ```
    pub fn sharpen(self) -> SharpeningBuilder {
        SharpeningBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_image_creation() {
        let img = RgbImage::new(100, 100);
        let sharpy_img = Image::from_rgb(img);
        assert_eq!(sharpy_img.dimensions(), (100, 100));
    }
    
    #[test]
    fn test_parameter_validation() {
        // Test unsharp mask
        let img1 = RgbImage::new(100, 100);
        let sharpy_img1 = Image::from_rgb(img1);
        assert!(sharpy_img1.unsharp_mask(-1.0, 1.0, 0).is_err());
        
        // Test high pass sharpen
        let img2 = RgbImage::new(100, 100);
        let sharpy_img2 = Image::from_rgb(img2);
        assert!(sharpy_img2.high_pass_sharpen(-1.0).is_err());
        
        // Test enhance edges
        let img3 = RgbImage::new(100, 100);
        let sharpy_img3 = Image::from_rgb(img3);
        assert!(sharpy_img3.enhance_edges(-1.0, EdgeMethod::Sobel).is_err());
        
        // Test clarity
        let img4 = RgbImage::new(100, 100);
        let sharpy_img4 = Image::from_rgb(img4);
        assert!(sharpy_img4.clarity(-1.0, 1.0).is_err());
    }
}