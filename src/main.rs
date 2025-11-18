extern crate alloc;

mod app;
mod labelary;
mod printer;
mod printer_status;
mod zpl;

use nightshade::prelude::*;

use app::Zebras;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Zebras::default())?;
    Ok(())
}
