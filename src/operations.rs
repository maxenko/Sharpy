//! Common operation types used throughout the library and CLI.

use crate::EdgeMethod;

/// Represents a sharpening operation that can be applied to an image.
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    /// Unsharp mask operation
    UnsharpMask { 
        /// Blur radius (0.5-10.0)
        radius: f32, 
        /// Strength amount (0.0-5.0)
        amount: f32, 
        /// Threshold (0-255)
        threshold: u8 
    },
    /// High-pass sharpening
    HighPassSharpen { 
        /// Strength (0.0-3.0)
        strength: f32 
    },
    /// Edge enhancement
    EnhanceEdges { 
        /// Strength (0.0-3.0)
        strength: f32, 
        /// Edge detection method
        method: EdgeMethod 
    },
    /// Clarity enhancement
    Clarity { 
        /// Strength (0.0-3.0)
        strength: f32, 
        /// Radius (1.0-20.0)
        radius: f32 
    },
}

impl Operation {
    /// Get a human-readable name for the operation
    pub fn name(&self) -> &'static str {
        match self {
            Operation::UnsharpMask { .. } => "Unsharp Mask",
            Operation::HighPassSharpen { .. } => "High-Pass Sharpen",
            Operation::EnhanceEdges { .. } => "Edge Enhancement",
            Operation::Clarity { .. } => "Clarity",
        }
    }
}