use anyhow::{Context, Result};
use gdal::Dataset;
use std::path::Path;
use gdal::raster::Buffer;

/// Will change this to an Array2<f64>
pub struct OverviewData {
    pub buffer: Buffer<f64>,
    pub width: usize,
    pub height: usize,
}

impl OverviewData {
    /// A helper to get the data as a slice, mimicking the .data() behavior
    pub fn data(&self) -> &[f64] {
        self.buffer.data()
    }
}

/// Orchestrates the entire process: expansion, opening, and reading
pub fn get_raster_overview(
    raw_path: &str,
    band_index: usize,
    level_index: usize,
) -> Result<OverviewData> {
    let expanded_path = shellexpand::tilde(raw_path).into_owned();
    let path = Path::new(&expanded_path);

    let dataset = Dataset::open(path)
        .with_context(|| format!("Failed to open GDAL dataset at {:?}", path))?;

    let main_band = dataset
        .rasterband(band_index)
        .with_context(|| format!("Could not find band {}", band_index))?;

    // 4. Access the requested overview level
    let ov_count = main_band.overview_count()? as usize;
    if level_index >= ov_count {
        anyhow::bail!(
            "Requested overview index {} but band only has {} overviews.",
            level_index,
            ov_count
        );
    }

    let overview_band = main_band
        .overview(level_index)
        .context("Failed to access overview band")?;

    let (ov_w, ov_h) = overview_band.size();

    // 5. Read the data
    // Per your previous note, we access the buffer content here.
    let rv = overview_band.read_as::<f64>(
        (0, 0),
        (ov_w, ov_h),
        (ov_w, ov_h),
        None,
    )?;

    Ok(OverviewData {
        // Depending on your specific gdal version, this might be .data or .buffer()
        buffer: rv, 
        width: ov_w,
        height: ov_h,
    })
}
