use anyhow::{bail, Result};
use image::imageops::FilterType;
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};

#[derive(Debug, Clone, Copy)]
pub enum Layout {
    Horizontal,
    Vertical,
    Grid { cols: usize, rows: usize },
}

#[derive(Debug, Clone)]
pub struct CombineConfig {
    pub layout: Layout,
    pub gap: u32,
    pub bg: [u8; 4],
    pub order: Option<Vec<usize>>,
    pub cell_width: Option<u32>,
    pub cell_height: Option<u32>,
}

pub fn parse_hex_color(s: &str) -> Result<[u8; 4]> {
    let s = s.trim().trim_start_matches('#');
    match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16)?;
            let g = u8::from_str_radix(&s[2..4], 16)?;
            let b = u8::from_str_radix(&s[4..6], 16)?;
            Ok([r, g, b, 255])
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16)?;
            let g = u8::from_str_radix(&s[2..4], 16)?;
            let b = u8::from_str_radix(&s[4..6], 16)?;
            let a = u8::from_str_radix(&s[6..8], 16)?;
            Ok([r, g, b, a])
        }
        _ => bail!("Color must be RRGGBB or RRGGBBAA"),
    }
}

pub fn apply_order(images: Vec<DynamicImage>, order: Option<&Vec<usize>>) -> Result<Vec<DynamicImage>> {
    if let Some(order) = order {
        if images.is_empty() {
            bail!("No input images provided");
        }

        let mut out = Vec::with_capacity(order.len());
        for &idx in order {
            let img = images
                .get(idx)
                .ok_or_else(|| anyhow::anyhow!("Order index {} out of range 0..{}", idx, images.len().saturating_sub(1)))?;
            out.push(img.clone());
        }
        Ok(out)
    } else {
        Ok(images)
    }
}

fn paste(canvas: &mut RgbaImage, img: &RgbaImage, x: u32, y: u32) {
    for iy in 0..img.height() {
        for ix in 0..img.width() {
            let px = img.get_pixel(ix, iy);
            canvas.put_pixel(x + ix, y + iy, *px);
        }
    }
}

fn resized(img: &DynamicImage, w: u32, h: u32) -> RgbaImage {
    img.resize_exact(w, h, FilterType::Lanczos3).to_rgba8()
}

pub fn combine(images: Vec<DynamicImage>, config: &CombineConfig) -> Result<RgbaImage> {
    let images = apply_order(images, config.order.as_ref())?;
    if images.is_empty() {
        bail!("No images to combine");
    }

    let gap = config.gap;
    let bg = Rgba(config.bg);

    match config.layout {
        Layout::Horizontal => {
            let cell_h = config
                .cell_height
                .unwrap_or_else(|| images.iter().map(|i| i.height()).max().unwrap_or(1));
            let resized_imgs: Vec<RgbaImage> = images
                .iter()
                .map(|img| {
                    let h = cell_h;
                    let w = config.cell_width.unwrap_or_else(|| {
                        let ratio = img.width() as f32 / img.height() as f32;
                        ((h as f32) * ratio).round().max(1.0) as u32
                    });
                    resized(img, w, h)
                })
                .collect();

            let total_w: u32 = resized_imgs.iter().map(|i| i.width()).sum::<u32>()
                + gap * resized_imgs.len().saturating_sub(1) as u32;
            let total_h: u32 = resized_imgs.iter().map(|i| i.height()).max().unwrap_or(1);

            let mut canvas: RgbaImage = ImageBuffer::from_pixel(total_w, total_h, bg);
            let mut x = 0;
            for img in &resized_imgs {
                paste(&mut canvas, img, x, 0);
                x += img.width() + gap;
            }
            Ok(canvas)
        }

        Layout::Vertical => {
            let cell_w = config
                .cell_width
                .unwrap_or_else(|| images.iter().map(|i| i.width()).max().unwrap_or(1));
            let resized_imgs: Vec<RgbaImage> = images
                .iter()
                .map(|img| {
                    let w = cell_w;
                    let h = config.cell_height.unwrap_or_else(|| {
                        let ratio = img.height() as f32 / img.width() as f32;
                        ((w as f32) * ratio).round().max(1.0) as u32
                    });
                    resized(img, w, h)
                })
                .collect();

            let total_w: u32 = resized_imgs.iter().map(|i| i.width()).max().unwrap_or(1);
            let total_h: u32 = resized_imgs.iter().map(|i| i.height()).sum::<u32>()
                + gap * resized_imgs.len().saturating_sub(1) as u32;

            let mut canvas: RgbaImage = ImageBuffer::from_pixel(total_w, total_h, bg);
            let mut y = 0;
            for img in &resized_imgs {
                paste(&mut canvas, img, 0, y);
                y += img.height() + gap;
            }
            Ok(canvas)
        }

        Layout::Grid { cols, rows } => {
            if cols == 0 || rows == 0 {
                bail!("Grid rows and cols must be greater than 0");
            }

            let cell_w = config
                .cell_width
                .unwrap_or_else(|| images.iter().map(|i| i.width()).max().unwrap_or(1));
            let cell_h = config
                .cell_height
                .unwrap_or_else(|| images.iter().map(|i| i.height()).max().unwrap_or(1));

            let total_w = cols as u32 * cell_w + gap * cols.saturating_sub(1) as u32;
            let total_h = rows as u32 * cell_h + gap * rows.saturating_sub(1) as u32;

            let mut canvas: RgbaImage = ImageBuffer::from_pixel(total_w, total_h, bg);

            for (i, img) in images.iter().take(cols * rows).enumerate() {
                let col = i % cols;
                let row = i / cols;
                let x = col as u32 * (cell_w + gap);
                let y = row as u32 * (cell_h + gap);
                let img = resized(img, cell_w, cell_h);
                paste(&mut canvas, &img, x, y);
            }

            Ok(canvas)
        }
    }
}