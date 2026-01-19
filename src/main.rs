use anyhow::Result;
use ndarray::s;

// Module
mod readband;
mod overview;
mod reader;

fn main() -> Result<()> {
    // Only the raw_path stays here
    let raw_path = "~/Documents/Codes/Experiments/data/test_rast.tif";

    let step: usize = 20;

    // Call the processor (Band 1, Overview index 3)
    match overview::read_overview(raw_path, 1, 3) {
        Ok(array) => {
            println!("Success! Loaded {:?} pixels.", array.shape());
            let (w, h): (usize, usize) = (array.shape()[0] / 2, array.shape()[1] / 2);
            let (w2, h2) = (w + step, h + step);
            println!("Sample: {:?}", &array.slice(s![w..w2, h..h2]));
        }
        Err(e) => {
            eprintln!("Error processing raster: {}", e);
        }
    }

    Ok(())
}