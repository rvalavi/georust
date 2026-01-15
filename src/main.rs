use anyhow::{Context, Result};
use gdal::Dataset;
use std::path::Path;

fn main() -> Result<()> {
    // 1. Fix the path: macOS uses /Users/ instead of /home/
    // Use a more robust expansion and handle the potential error
    let raw_path = "~/Documents/Codes/Experiments/data/ACC50_85.tif";
    let expanded_path = shellexpand::tilde(raw_path).into_owned();
    let path = Path::new(&expanded_path);

    // 2. Open the dataset with context for better error messages
    let dataset = Dataset::open(path)
        .with_context(|| format!("Failed to open dataset at {:?}", path))?;

    println!("--- Dataset Info ---");
    println!("Driver: {}", dataset.driver().short_name());
    println!("Raster count: {}", dataset.raster_count());
    
    // Note: layer_count() refers to Vector layers (OGR). 
    // Most .tif files will return 0 here.
    println!("Vector Layer count: {}", dataset.layer_count());

    // 3. Work with Raster Bands
    if dataset.raster_count() > 0 {
        // GDAL uses 1-based indexing for bands
        let rasterband = dataset.rasterband(1)?; 
        
        let (width, height) = rasterband.size();
        println!("\n--- Band 1 Details ---");
        println!("  Size: {}x{}", width, height);
        println!("  Block size: {:?}", rasterband.block_size());
        println!("  Data type: {:?}", rasterband.band_type());
        
        // 4. Read data (First 10x10 pixels)
        let read_width = width.min(10);
        let read_height = height.min(10);
        
        // Using read_as::<f64> correctly handles the buffer type
        let buffer = rasterband.read_as::<f32>(
            (0, 0),             // (x_offset, y_offset)
            (read_width, read_height),   // (window_width, window_height)
            (read_width, read_height),   // (buffer_width, buffer_height)
            None,               // No extra arguments
        )?;
        
        println!("  First few pixel values: {:?}", &buffer.data()[..10.min(buffer.data().len())]);
    } else {
        println!("No raster bands found in this file.");
    }

    Ok(())
}


