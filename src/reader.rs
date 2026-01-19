use anyhow::{Context, Result};
use gdal::raster::{GdalType, RasterBand, ResampleAlg};
use ndarray::Array2;


/// Read a window from a RasterBand into an ndarray::Array2.
#[inline]
pub fn read_as_array<T>(
    band: &RasterBand,
    window: (isize, isize),
    window_size: (usize, usize),
    array_size: (usize, usize),
    resample_alg: Option<ResampleAlg>,
) -> Result<Array2<T>>
where
    T: Copy + GdalType,
{
    // read_as returns Result<Buffer<T>, GdalError>
    let buf = band.read_as::<T>(window, window_size, array_size, resample_alg)
        .context("GDAL failed to read band into buffer")?;

    // In gdal 0.19+, 'buf.data' is the public Vec field.
    let owned_pixels = buf.data().to_vec();

    // GDAL is (width, height), but ndarray expects (rows, cols) -> (height, width).
    let shape = (array_size.1, array_size.0);
    
    let array = Array2::from_shape_vec(shape, owned_pixels)
        .map_err(|e| anyhow::anyhow!("NDArray shape mismatch: {}", e))?;

    Ok(array)
}


/// Convenience to read the entire band into an Array2.
#[inline]
pub fn read_full_band<T>(band: &RasterBand) -> Result<Array2<T>>
where
    T: Copy + GdalType,
{
    let size = band.size();      // (cols, rows)
    read_as_array(band, (0, 0), size, size, None)
}
