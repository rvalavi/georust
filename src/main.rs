use anyhow::Result;

// Module
mod readband;
mod overview;

fn main() -> Result<()> {
    // Only the raw_path stays here
    let raw_path = "~/Documents/Codes/Experiments/data/condition.tif";

    // Call the processor (Band 1, Overview index 3)
    match overview::get_raster_overview(raw_path, 1, 3) {
        Ok(data) => {
            println!("Success! Loaded {}x{} pixels.", data.width, data.height);
            println!("Sample: {:?}", &data.data()[..10.min(data.data().len())]);
        }
        Err(e) => {
            eprintln!("Error processing raster: {}", e);
        }
    }

    Ok(())
}