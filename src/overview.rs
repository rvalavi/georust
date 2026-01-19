use gdal::Dataset;
use gdal::raster::{RasterBand, ResampleAlg};
use ndarray::Array2;
use anyhow::Result;
use std::path::Path;

use crate::reader::read_as_array;

pub fn read_overview(raw_path: &str, band_index: usize, ovr_index: usize) -> Result<Array2<f64>> {
    let expanded_path = shellexpand::tilde(raw_path).into_owned();
    let path = Path::new(&expanded_path);
    // open the dataset and retrieve the full‑resolution band (1‑based index)
    let ds = Dataset::open(path)?;
    let band = ds.rasterband(band_index)?;

    // check overview count and obtain the overview band
    let ovr_count = band.overview_count()? as usize;
    if ovr_index >= ovr_count {
        anyhow::bail!("overview index {} out of range ({} available)", ovr_index, ovr_count);
    }
    let overview_band: RasterBand = band.overview(ovr_index)?;

    // dimensions of the overview band
    let (width, height) = (overview_band.x_size() as usize, overview_band.y_size() as usize);

    // read the entire overview into an ndarray using ChunkReader::read_as_array
    let arr: Array2<f64> = read_as_array(
        &overview_band, 
        (0, 0), 
        (width, height),
        (width, height),
        Some(ResampleAlg::Average),
    )?;

    Ok(arr)
}
