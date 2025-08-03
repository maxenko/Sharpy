//! Example showing how to use built-in sharpening presets

use sharpy::{Image, SharpeningPresets};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Load an image
    let image = Image::load("tests/fixtures/lens.jpg")?;
    
    // Try different presets
    let presets = [
        ("subtle", |img: Image| SharpeningPresets::subtle(img).apply()),
        ("moderate", |img: Image| SharpeningPresets::moderate(img).apply()),
        ("strong", |img: Image| SharpeningPresets::strong(img).apply()),
        ("portrait", |img: Image| SharpeningPresets::portrait(img).apply()),
        ("landscape", |img: Image| SharpeningPresets::landscape(img).apply()),
        ("edge_aware", |img: Image| SharpeningPresets::edge_aware(img).apply()),
    ];
    
    for (name, preset_fn) in presets {
        println!("Applying {} preset...", name);
        
        let result = preset_fn(image.clone())?;
        let output_path = format!("examples/output/lens_{}.jpg", name);
        
        result.save(&output_path)?;
        println!("  Saved to {}", output_path);
    }
    
    Ok(())
}