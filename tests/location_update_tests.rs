use corelocation::prelude::*;

#[test]
fn location_updater_creation_and_invalidation_smoke() -> Result<(), Box<dyn std::error::Error>> {
    if LocationUpdater::is_supported() {
        let updater = LocationUpdater::new()?;
        updater.invalidate();
    }
    Ok(())
}
