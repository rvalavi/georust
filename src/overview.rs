use gdal::Dataset;
use gdal::raster::{RasterBand, ResampleAlg};
use ndarray::{Array2, Array3, Axis, stack};
use rayon::prelude::*; // Import Rayon traits
use anyhow::{Context, Result};
use std::path::Path;
// helper mod
use crate::reader::read_as_array;


pub fn read_multiband_parallel(
    raw_path: &str,
    window: (isize, isize),
    window_size: (usize, usize),
    array_size: (usize, usize),
) -> Result<Array3<f64>> {
    // 1. Open once on the main thread just to get the band count
    let band_count = {
        let path = shellexpand::tilde(raw_path).into_owned();
        let ds = Dataset::open(Path::new(&path))?;
        ds.raster_count()
    };

    // 2. Use Rayon to iterate over band indices in parallel
    // We pass the raw_path string to the closures
    let bands_data: Vec<Array2<f64>> = (1..=band_count)
        .into_par_iter() // <--- Parallel Iterator
        .map(|i| {
            // A. Each thread opens its own Dataset handle
            let path = shellexpand::tilde(raw_path).into_owned();
            let ds = Dataset::open(Path::new(&path))
                .with_context(|| format!("Thread failed to open {}", path))?;

            // B. Get the specific band for this thread
            let band = ds.rasterband(i)?;

            // C. Read the data (using your existing logic)
            read_as_array(
                &band,
                window,
                window_size,
                array_size,
                Some(ResampleAlg::Average),
            )
        })
        .collect::<Result<Vec<_>>>()?; // Collects results and handles errors

    // 3. Stack the results on the main thread
    let views: Vec<_> = bands_data.iter().map(|a| a.view()).collect();
    
    let arr_3d = stack(Axis(0), &views)
        .map_err(|e| anyhow::anyhow!("Failed to stack parallel bands: {}", e))?;

    Ok(arr_3d)
}


pub fn read_full_dataset_3d(
    ds: &Dataset,
    window: (isize, isize),
    window_size: (usize, usize),
    array_size: (usize, usize),
) -> Result<Array3<f64>> {
    let band_count = ds.raster_count() as usize;
    
    // Dataset::read_raster can read multiple bands into one buffer
    // The bands are read sequentially into the buffer
    let bands: Vec<isize> = (1..=band_count as isize).collect();
    
    let buf = ds.read_raster::<f64>(
        window,
        window_size,
        array_size,
        &bands,
        None, // No extra args
    )?;

    // Shape: (Bands, Height, Width)
    let shape = (band_count, array_size.1, array_size.0);
    
    let arr_3d = Array3::from_shape_vec(shape, buf.data().to_vec())
        .map_err(|e| anyhow::anyhow!("3D Shape mismatch: {}", e))?;

    Ok(arr_3d)
}


pub fn read_multiband_3d(
    ds: &Dataset,
    window: (isize, isize),
    window_size: (usize, usize),
    array_size: (usize, usize),
) -> Result<Array3<f64>> {
    let band_count = ds.raster_count() as usize;
    let mut bands = Vec::with_capacity(band_count);

    for i in 1..=band_count {
        let band = ds.rasterband(i)?;
        // Reuse your existing read_as_array function
        let arr_2d = read_as_array::<f64>(
            &band,
            window,
            window_size,
            array_size,
            Some(ResampleAlg::Average),
        )?;
        bands.push(arr_2d);
    }

    // Convert Vec<Array2> to Vec<ArrayView2> for stacking
    let views: Vec<_> = bands.iter().map(|a| a.view()).collect();
    
    // Stack along Axis(0) to get shape (Bands, Height, Width)
    let arr_3d = stack(Axis(0), &views)
        .map_err(|e| anyhow::anyhow!("Failed to stack bands: {}", e))?;

    Ok(arr_3d)
}


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
