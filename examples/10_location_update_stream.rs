use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "location_updates_supported = {}",
        LocationUpdater::is_supported()
    );
    if LocationUpdater::is_supported() {
        let updater = LocationUpdater::new()?;
        updater.invalidate();
        println!("location updater created and invalidated");
    }
    Ok(())
}
