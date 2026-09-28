//! Cropping a rejilla (grid) image into individual sub-images — the
//! mechanical half of Modo 2 (`odd/tasks/ai-image-grid-generation.md`).
//!
//! Rust never judges whether a crop is coherent with its concept (that's
//! always the agent, per this feature's architecture principle); this
//! module only does the mechanical division of one image into R×C images
//! in reading order.

use image::{DynamicImage, GenericImageView};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Crops `img` into `rows` x `cols` equal-sized sub-images, in reading order
/// (row 0 left-to-right, then row 1 left-to-right, ...): the returned
/// `Vec`'s index `i` is `row * cols + col`, so `crops[0]` is the top-left
/// cell and `crops[cols]` is the first cell of the second row.
///
/// Cell size is computed with truncated integer division
/// (`width / cols`, `height / rows`). When the image's dimensions aren't
/// evenly divisible by `cols`/`rows`, every cell still uses that same
/// truncated size — the leftover strip of at most `cols - 1` pixels wide
/// and `rows - 1` pixels tall (bottom/right edge of the source image) is
/// simply never included in any crop, rather than being distributed
/// unevenly onto the last row/column or causing an error. This keeps every
/// cell the same size (simpler for a caller assigning crops to concepts)
/// and the dropped strip is negligible for a real grid image generated at
/// a size the caller chose to be evenly divisible in the first place.
pub fn crop_grid(img: &DynamicImage, rows: u32, cols: u32) -> Vec<DynamicImage> {
    let (width, height) = img.dimensions();
    let cell_w = width / cols.max(1);
    let cell_h = height / rows.max(1);

    let mut crops = Vec::with_capacity((rows * cols) as usize);
    for row in 0..rows {
        for col in 0..cols {
            let x = col * cell_w;
            let y = row * cell_h;
            crops.push(img.crop_imm(x, y, cell_w, cell_h));
        }
    }
    crops
}

/// Writes `crops` (already in reading order) to `out_dir` as
/// `crop-01.png`..`crop-<N>.png` (2-digit zero-padded), creating `out_dir`
/// if it doesn't exist. Returns the written paths, in the same reading
/// order as `crops`.
pub fn write_crops(crops: &[DynamicImage], out_dir: &Path) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(out_dir)?;

    let mut paths = Vec::with_capacity(crops.len());
    for (index, crop) in crops.iter().enumerate() {
        let path = out_dir.join(format!("crop-{:02}.png", index + 1));
        crop.save_with_format(&path, image::ImageFormat::Png)
            .map_err(io::Error::other)?;
        paths.push(path);
    }
    Ok(paths)
}

/// Loads `file`, crops it into `rows` x `cols` sub-images, and writes them
/// to `out_dir` — the full mechanical pipeline behind
/// `learnkit cards image-grid crop`.
pub fn crop_file_to_dir(
    file: &Path,
    rows: u32,
    cols: u32,
    out_dir: &Path,
) -> io::Result<Vec<PathBuf>> {
    let img = image::open(file).map_err(io::Error::other)?;
    let crops = crop_grid(&img, rows, cols);
    write_crops(&crops, out_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    /// Builds a 400x400 test image with 16 distinctly-colored 100x100
    /// quadrants (a 4x4 grid), laid out in reading order: quadrant index
    /// `row * 4 + col` gets color `(index * 16, 255 - index * 16, 128, 255)`
    /// — a cheap, deterministic, visually-distinct palette good enough to
    /// spot-check "this crop came from the right grid position."
    fn sixteen_quadrant_image() -> RgbaImage {
        let mut img = RgbaImage::new(400, 400);
        for row in 0..4u32 {
            for col in 0..4u32 {
                let index = row * 4 + col;
                let color = Rgba([
                    (index * 16) as u8,
                    255u8.saturating_sub((index * 16) as u8),
                    128,
                    255,
                ]);
                for y in (row * 100)..(row * 100 + 100) {
                    for x in (col * 100)..(col * 100 + 100) {
                        img.put_pixel(x, y, color);
                    }
                }
            }
        }
        img
    }

    fn expected_color(index: u32) -> Rgba<u8> {
        Rgba([
            (index * 16) as u8,
            255u8.saturating_sub((index * 16) as u8),
            128,
            255,
        ])
    }

    #[test]
    fn crop_grid_produces_16_crops_in_reading_order_with_the_right_pixels() {
        let img = DynamicImage::ImageRgba8(sixteen_quadrant_image());

        let crops = crop_grid(&img, 4, 4);

        assert_eq!(crops.len(), 16);
        for (index, crop) in crops.iter().enumerate() {
            assert_eq!(crop.width(), 100);
            assert_eq!(crop.height(), 100);
            // Spot-check a pixel near the middle of the crop — the
            // strongest regression check that reading order is correct:
            // if row/col math were swapped or transposed, this would pick
            // up the wrong quadrant's color.
            let pixel = crop.get_pixel(50, 50);
            assert_eq!(
                pixel,
                expected_color(index as u32),
                "crop {index} has the wrong color (reading-order bug?)"
            );
        }
    }

    #[test]
    fn crop_grid_truncates_cell_size_when_not_evenly_divisible() {
        let img = DynamicImage::ImageRgba8(RgbaImage::new(401, 401));

        let crops = crop_grid(&img, 4, 4);

        assert_eq!(crops.len(), 16);
        for crop in &crops {
            // 401 / 4 == 100 (truncated); the extra 1px strip is dropped.
            assert_eq!(crop.width(), 100);
            assert_eq!(crop.height(), 100);
        }
    }

    #[test]
    fn write_crops_names_files_in_reading_order_zero_padded() {
        let dir = tempfile::tempdir().unwrap();
        let img = DynamicImage::ImageRgba8(sixteen_quadrant_image());
        let crops = crop_grid(&img, 4, 4);

        let paths = write_crops(&crops, dir.path()).unwrap();

        assert_eq!(paths.len(), 16);
        assert_eq!(paths[0].file_name().unwrap(), "crop-01.png");
        assert_eq!(paths[1].file_name().unwrap(), "crop-02.png");
        assert_eq!(paths[4].file_name().unwrap(), "crop-05.png");
        assert_eq!(paths[15].file_name().unwrap(), "crop-16.png");
        for path in &paths {
            assert!(path.exists());
        }
    }

    #[test]
    fn crop_file_to_dir_creates_the_output_directory() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("rejilla.png");
        let img = DynamicImage::ImageRgba8(sixteen_quadrant_image());
        img.save_with_format(&source_path, image::ImageFormat::Png)
            .unwrap();
        let out_dir = dir.path().join("nested").join("crops");

        let paths = crop_file_to_dir(&source_path, 4, 4, &out_dir).unwrap();

        assert_eq!(paths.len(), 16);
        assert!(out_dir.exists());
    }
}
