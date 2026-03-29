#![deny(warnings)]

mod combiner;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use combiner::{combine, parse_hex_color, CombineConfig, Layout};
use glob::glob;
use image::DynamicImage;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(
    name = "imgmerge",
    version,
    about = "Combine multiple images horizontally, vertically, or in a custom grid"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Stack images left-to-right in a single row
    Horizontal(CommonArgs),

    /// Stack images top-to-bottom in a single column
    Vertical(CommonArgs),

    /// Arrange images in a custom grid
    Grid(GridArgs),
}

#[derive(Args, Debug)]
struct CommonArgs {
    /// Input image paths or glob patterns (*.jpg on Windows supported)
    #[arg(required = true)]
    inputs: Vec<String>,

    /// Output image path
    #[arg(short, long)]
    output: String,

    /// Gap between images in pixels
    #[arg(long, default_value_t = 0)]
    gap: u32,

    /// Background color in hex: RRGGBB or RRGGBBAA
    #[arg(long, default_value = "00000000")]
    bg: String,

    /// Reorder images using 0-based indices, e.g. 2,0,1
    #[arg(long)]
    order: Option<String>,

    /// Force each image width
    #[arg(long)]
    cell_width: Option<u32>,

    /// Force each image height
    #[arg(long)]
    cell_height: Option<u32>,
}

#[derive(Args, Debug)]
struct GridArgs {
    /// Input image paths or glob patterns (*.jpg on Windows supported)
    #[arg(required = true)]
    inputs: Vec<String>,

    /// Output image path
    #[arg(short, long)]
    output: String,

    /// Grid columns
    #[arg(long)]
    cols: usize,

    /// Optional grid rows; if omitted, computed automatically
    #[arg(long)]
    rows: Option<usize>,

    /// Gap between cells in pixels
    #[arg(long, default_value_t = 0)]
    gap: u32,

    /// Background color in hex: RRGGBB or RRGGBBAA
    #[arg(long, default_value = "00000000")]
    bg: String,

    /// Reorder images using 0-based indices, e.g. 5,0,1,2,3,4
    #[arg(long)]
    order: Option<String>,

    /// Force each cell width
    #[arg(long)]
    cell_width: Option<u32>,

    /// Force each cell height
    #[arg(long)]
    cell_height: Option<u32>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Horizontal(args) => run_common(args, Layout::Horizontal),
        Commands::Vertical(args) => run_common(args, Layout::Vertical),
        Commands::Grid(args) => run_grid(args),
    }
}

fn run_common(args: CommonArgs, layout: Layout) -> Result<()> {
    let inputs = expand_globs(&args.inputs)?;
    let inputs = exclude_output(inputs, &args.output);
    let images = load_images(&inputs)?;
    let bg = parse_hex_color(&args.bg)?;
    let order = parse_order(args.order.as_deref())?;

    let config = CombineConfig {
        layout,
        gap: args.gap,
        bg,
        order,
        cell_width: args.cell_width,
        cell_height: args.cell_height,
    };

    let out = combine(images, &config)?;
    save_output(&out, &args.output)?;
    Ok(())
}

fn run_grid(args: GridArgs) -> Result<()> {
    if args.cols == 0 {
        bail!("--cols must be greater than 0");
    }

    let inputs = expand_globs(&args.inputs)?;
    let inputs = exclude_output(inputs, &args.output);
    let images = load_images(&inputs)?;
    let bg = parse_hex_color(&args.bg)?;
    let order = parse_order(args.order.as_deref())?;

    let image_count = order.as_ref().map(|o| o.len()).unwrap_or(images.len());
    if image_count == 0 {
        bail!("No input images");
    }

    let rows = args.rows.unwrap_or_else(|| image_count.div_ceil(args.cols));

    let config = CombineConfig {
        layout: Layout::Grid {
            cols: args.cols,
            rows,
        },
        gap: args.gap,
        bg,
        order,
        cell_width: args.cell_width,
        cell_height: args.cell_height,
    };

    let out = combine(images, &config)?;
    save_output(&out, &args.output)?;
    Ok(())
}

fn expand_globs(inputs: &[String]) -> Result<Vec<String>> {
    let mut paths = Vec::new();

    for input in inputs {
        if input.contains('*') || input.contains('?') || input.contains('[') {
            let mut matched_any = false;

            for entry in glob(input)
                .with_context(|| format!("Invalid glob pattern '{}'", input))?
            {
                let path = entry?;
                matched_any = true;
                paths.push(path.to_string_lossy().into_owned());
            }

            if !matched_any {
                bail!("No files matched pattern '{}'", input);
            }
        } else {
            paths.push(input.clone());
        }
    }

    if paths.is_empty() {
        bail!("No input files found");
    }

    paths.sort();
    Ok(paths)
}

/// Remove the output path from the expanded input list.
/// This prevents *.jpeg from accidentally matching the output file itself.
fn exclude_output(inputs: Vec<String>, output: &str) -> Vec<String> {
    let output_canonical = std::fs::canonicalize(output).ok();

    inputs
        .into_iter()
        .filter(|input| {
            if let Some(ref out_canon) = output_canonical {
                // Output file already exists — compare by canonical path
                match std::fs::canonicalize(input) {
                    Ok(inp_canon) => inp_canon != *out_canon,
                    Err(_) => true,
                }
            } else {
                // Output doesn't exist yet — compare normalised strings
                let norm_in  = input.replace('\\', "/");
                let norm_out = output.replace('\\', "/");
                norm_in != norm_out
            }
        })
        .collect()
}

fn load_images(inputs: &[String]) -> Result<Vec<DynamicImage>> {
    let mut images = Vec::with_capacity(inputs.len());

    for input in inputs {
        let img = image::open(input)
            .with_context(|| format!("Could not open '{}'", input))?;
        images.push(img);
    }

    Ok(images)
}

fn parse_order(order: Option<&str>) -> Result<Option<Vec<usize>>> {
    match order {
        None => Ok(None),
        Some(s) => {
            let parsed = s
                .split(',')
                .filter(|p| !p.trim().is_empty())
                .map(|p| {
                    p.trim()
                        .parse::<usize>()
                        .with_context(|| format!("Invalid order index '{}'", p))
                })
                .collect::<Result<Vec<_>>>()?;

            if parsed.is_empty() {
                bail!("--order was provided but no indices were parsed");
            }

            Ok(Some(parsed))
        }
    }
}

fn save_output(img: &image::RgbaImage, output: &str) -> Result<()> {
    let path = Path::new(output);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // JPEG does not support alpha — flatten RGBA → RGB before saving
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "jpg" || ext == "jpeg" {
        let rgb = image::DynamicImage::ImageRgba8(img.clone()).into_rgb8();
        rgb.save(path)
            .with_context(|| format!("Could not save output '{}'", output))?;
    } else {
        img.save(path)
            .with_context(|| format!("Could not save output '{}'", output))?;
    }

    Ok(())
}