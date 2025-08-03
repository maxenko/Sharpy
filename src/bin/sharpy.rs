use clap::{Parser, Subcommand};
use sharpy::{Image, EdgeMethod, SharpeningPresets};
use anyhow::{Result, Context};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use glob::glob;

#[derive(Parser)]
#[command(name = "sharpy")]
#[command(author, version, about = "High-performance image sharpening tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    quiet: bool,
    
    /// Preview operations without processing
    #[arg(long, global = true)]
    dry_run: bool,
    
    /// Overwrite existing files without prompting
    #[arg(long, global = true)]
    overwrite: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply unsharp mask sharpening
    Unsharp {
        /// Input image file
        input: PathBuf,
        
        /// Output image file
        output: PathBuf,
        
        /// Blur radius (0.5-10.0)
        #[arg(short, long, default_value = "1.0")]
        radius: f32,
        
        /// Sharpening strength (0.0-5.0)
        #[arg(short, long, default_value = "1.0")]
        amount: f32,
        
        /// Minimum difference threshold (0-255)
        #[arg(short, long, default_value = "0")]
        threshold: u8,
    },
    
    /// Apply high-pass sharpening
    Highpass {
        /// Input image file
        input: PathBuf,
        
        /// Output image file
        output: PathBuf,
        
        /// Blend strength (0.0-3.0)
        #[arg(short, long, default_value = "0.5")]
        strength: f32,
    },
    
    /// Enhance edges in the image
    Edges {
        /// Input image file
        input: PathBuf,
        
        /// Output image file
        output: PathBuf,
        
        /// Enhancement strength (0.0-3.0)
        #[arg(short, long, default_value = "1.0")]
        strength: f32,
        
        /// Edge detection method
        #[arg(short, long, default_value = "sobel")]
        method: EdgeMethodArg,
    },
    
    /// Apply clarity enhancement
    Clarity {
        /// Input image file
        input: PathBuf,
        
        /// Output image file
        output: PathBuf,
        
        /// Enhancement strength (0.0-3.0)
        #[arg(short, long, default_value = "1.0")]
        strength: f32,
        
        /// Local area radius (1.0-20.0)
        #[arg(short, long, default_value = "2.0")]
        radius: f32,
    },
    
    /// Apply a sharpening preset
    Preset {
        /// Input image file
        input: PathBuf,
        
        /// Output image file
        output: PathBuf,
        
        /// Preset name
        #[arg(short, long)]
        preset: PresetArg,
    },
    
    /// Process multiple files with batch operations
    Batch {
        /// Input pattern (e.g., "*.jpg" or "images/*.png")
        pattern: String,
        
        /// Output directory
        #[arg(short, long)]
        output_dir: PathBuf,
        
        /// Output filename suffix
        #[arg(short, long, default_value = "_sharp")]
        suffix: String,
        
        /// Operations to apply (format: "operation:param1:param2:...")
        #[arg(short = 'p', long, value_delimiter = ',')]
        operations: Vec<String>,
    },
}

#[derive(Clone)]
enum EdgeMethodArg {
    Sobel,
    Prewitt,
}

impl From<EdgeMethodArg> for EdgeMethod {
    fn from(arg: EdgeMethodArg) -> Self {
        match arg {
            EdgeMethodArg::Sobel => EdgeMethod::Sobel,
            EdgeMethodArg::Prewitt => EdgeMethod::Prewitt,
        }
    }
}

impl std::str::FromStr for EdgeMethodArg {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sobel" => Ok(EdgeMethodArg::Sobel),
            "prewitt" => Ok(EdgeMethodArg::Prewitt),
            _ => Err(format!("Unknown edge method: {}. Use 'sobel' or 'prewitt'", s)),
        }
    }
}

#[derive(Clone)]
enum PresetArg {
    Subtle,
    Moderate,
    Strong,
    EdgeAware,
    Portrait,
    Landscape,
}

impl std::str::FromStr for PresetArg {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "subtle" => Ok(PresetArg::Subtle),
            "moderate" => Ok(PresetArg::Moderate),
            "strong" => Ok(PresetArg::Strong),
            "edge-aware" | "edge_aware" => Ok(PresetArg::EdgeAware),
            "portrait" => Ok(PresetArg::Portrait),
            "landscape" => Ok(PresetArg::Landscape),
            _ => Err(format!("Unknown preset: {}. Available: subtle, moderate, strong, edge-aware, portrait, landscape", s)),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Unsharp { input, output, radius, amount, threshold } => {
            process_single_image(&cli, input, output, |img| {
                img.unsharp_mask(*radius, *amount, *threshold)
            })
        }
        
        Commands::Highpass { input, output, strength } => {
            process_single_image(&cli, input, output, |img| {
                img.high_pass_sharpen(*strength)
            })
        }
        
        Commands::Edges { input, output, strength, method } => {
            let method = EdgeMethod::from(method.clone());
            process_single_image(&cli, input, output, |img| {
                img.enhance_edges(*strength, method)
            })
        }
        
        Commands::Clarity { input, output, strength, radius } => {
            process_single_image(&cli, input, output, |img| {
                img.clarity(*strength, *radius)
            })
        }
        
        Commands::Preset { input, output, preset } => {
            process_single_image(&cli, input, output, |img| {
                let builder = match preset {
                    PresetArg::Subtle => SharpeningPresets::subtle(img),
                    PresetArg::Moderate => SharpeningPresets::moderate(img),
                    PresetArg::Strong => SharpeningPresets::strong(img),
                    PresetArg::EdgeAware => SharpeningPresets::edge_aware(img),
                    PresetArg::Portrait => SharpeningPresets::portrait(img),
                    PresetArg::Landscape => SharpeningPresets::landscape(img),
                };
                builder.apply()
            })
        }
        
        Commands::Batch { pattern, output_dir, suffix, operations } => {
            process_batch(&cli, pattern, output_dir, suffix, operations)
        }
    }
}

fn process_single_image<F>(cli: &Cli, input: &Path, output: &Path, operation: F) -> Result<()>
where
    F: FnOnce(Image) -> sharpy::Result<Image>,
{
    if !cli.quiet {
        eprintln!("Processing: {} -> {}", input.display(), output.display());
    }
    
    // Check if output exists and handle overwrite
    if output.exists() && !cli.overwrite && !cli.dry_run {
        anyhow::bail!("Output file already exists: {}. Use --overwrite to replace.", output.display());
    }
    
    if cli.dry_run {
        if !cli.quiet {
            eprintln!("Dry run: Would process {} -> {}", input.display(), output.display());
        }
        return Ok(());
    }
    
    // Load image
    let image = Image::load(input)
        .with_context(|| format!("Failed to load image: {}", input.display()))?;
    
    if cli.verbose {
        let (width, height) = image.dimensions();
        eprintln!("Loaded image: {}x{}", width, height);
    }
    
    // Apply operation
    let result = operation(image)
        .map_err(|e| anyhow::anyhow!("Processing failed: {}", e))?;
    
    // Save result
    result.save(output)
        .with_context(|| format!("Failed to save image: {}", output.display()))?;
    
    if !cli.quiet {
        eprintln!("Successfully saved: {}", output.display());
    }
    
    Ok(())
}

fn process_batch(cli: &Cli, pattern: &str, output_dir: &Path, suffix: &str, operations: &[String]) -> Result<()> {
    // Parse operations
    let parsed_operations = parse_operations(operations)?;
    
    // Create output directory
    if !cli.dry_run {
        std::fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;
    }
    
    // Find matching files
    let files: Vec<_> = glob(pattern)
        .map_err(|e| anyhow::anyhow!("Invalid pattern: {}", e))?
        .filter_map(|entry| entry.ok())
        .collect();
    
    if files.is_empty() {
        anyhow::bail!("No files match pattern: {}", pattern);
    }
    
    if !cli.quiet {
        eprintln!("Found {} files to process", files.len());
    }
    
    // Setup progress bar
    let pb = if !cli.quiet {
        let pb = ProgressBar::new(files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
                .progress_chars("#>-")
        );
        Some(pb)
    } else {
        None
    };
    
    // Process each file
    let mut success_count = 0;
    let mut error_count = 0;
    
    for path in files {
        if let Some(pb) = &pb {
            pb.set_message(format!("Processing: {}", path.file_name().unwrap_or_default().to_string_lossy()));
        }
        
        // Generate output filename
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", path.display()))?;
        
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg");
        
        let output_filename = format!("{}{}.{}", stem, suffix, extension);
        let output_path = output_dir.join(output_filename);
        
        // Process the file
        let result = process_single_with_operations(cli, &path, &output_path, &parsed_operations);
        
        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                if !cli.quiet {
                    eprintln!("Error processing {}: {}", path.display(), e);
                }
            }
        }
        
        if let Some(pb) = &pb {
            pb.inc(1);
        }
    }
    
    if let Some(pb) = &pb {
        pb.finish_with_message(format!("Completed: {} successful, {} errors", success_count, error_count));
    }
    
    if error_count > 0 {
        anyhow::bail!("{} files failed to process", error_count);
    }
    
    Ok(())
}

enum Operation {
    Unsharp { radius: f32, amount: f32, threshold: u8 },
    Highpass { strength: f32 },
    Edges { strength: f32, method: EdgeMethod },
    Clarity { strength: f32, radius: f32 },
}

fn parse_operations(operations: &[String]) -> Result<Vec<Operation>> {
    operations.iter()
        .map(|op| parse_single_operation(op))
        .collect()
}

fn parse_single_operation(op: &str) -> Result<Operation> {
    let parts: Vec<&str> = op.split(':').collect();
    
    match parts.first().map(|s| s.to_lowercase()).as_deref() {
        Some("unsharp") => {
            if parts.len() != 4 {
                anyhow::bail!("Unsharp requires 3 parameters: unsharp:radius:amount:threshold");
            }
            Ok(Operation::Unsharp {
                radius: parts[1].parse().context("Invalid radius")?,
                amount: parts[2].parse().context("Invalid amount")?,
                threshold: parts[3].parse().context("Invalid threshold")?,
            })
        }
        Some("highpass") => {
            if parts.len() != 2 {
                anyhow::bail!("Highpass requires 1 parameter: highpass:strength");
            }
            Ok(Operation::Highpass {
                strength: parts[1].parse().context("Invalid strength")?,
            })
        }
        Some("edges") => {
            if parts.len() != 3 {
                anyhow::bail!("Edges requires 2 parameters: edges:strength:method");
            }
            let method = match parts[2].to_lowercase().as_str() {
                "sobel" => EdgeMethod::Sobel,
                "prewitt" => EdgeMethod::Prewitt,
                _ => anyhow::bail!("Unknown edge method: {}", parts[2]),
            };
            Ok(Operation::Edges {
                strength: parts[1].parse().context("Invalid strength")?,
                method,
            })
        }
        Some("clarity") => {
            if parts.len() != 3 {
                anyhow::bail!("Clarity requires 2 parameters: clarity:strength:radius");
            }
            Ok(Operation::Clarity {
                strength: parts[1].parse().context("Invalid strength")?,
                radius: parts[2].parse().context("Invalid radius")?,
            })
        }
        _ => anyhow::bail!("Unknown operation: {}", parts.first().unwrap_or(&"<empty>")),
    }
}

fn process_single_with_operations(cli: &Cli, input: &Path, output: &Path, operations: &[Operation]) -> Result<()> {
    if cli.dry_run {
        if cli.verbose {
            eprintln!("Dry run: Would process {} -> {} with {} operations", 
                     input.display(), output.display(), operations.len());
        }
        return Ok(());
    }
    
    // Load image
    let mut image = Image::load(input)
        .with_context(|| format!("Failed to load image: {}", input.display()))?;
    
    // Apply each operation in sequence
    for operation in operations {
        image = match operation {
            Operation::Unsharp { radius, amount, threshold } => {
                image.unsharp_mask(*radius, *amount, *threshold)
            }
            Operation::Highpass { strength } => {
                image.high_pass_sharpen(*strength)
            }
            Operation::Edges { strength, method } => {
                image.enhance_edges(*strength, *method)
            }
            Operation::Clarity { strength, radius } => {
                image.clarity(*strength, *radius)
            }
        }.map_err(|e| anyhow::anyhow!("Operation failed: {}", e))?;
    }
    
    // Save result
    image.save(output)
        .with_context(|| format!("Failed to save image: {}", output.display()))?;
    
    Ok(())
}