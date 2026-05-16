use std::time::{SystemTime, UNIX_EPOCH};

use corelocation::prelude::*;

fn unique_monitor_name() -> String {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("CoreLocationMonitor{suffix}")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = MonitorConfiguration::new(unique_monitor_name());
    match configuration.open() {
        Ok(monitor) => {
            let condition =
                CircularGeographicCondition::new(Coordinate::new(37.3349, -122.0090), 250.0)?;
            let identifier = "appleparkcircle";
            monitor.add_condition_assuming_state(
                &condition,
                identifier,
                MonitoringState::Satisfied,
            )?;
            println!("monitor_name = {}", monitor.name());
            println!("condition = {:?}", condition.snapshot()?);
            println!("identifiers = {:?}", monitor.monitored_identifiers()?);
            println!("record = {:?}", monitor.record(identifier)?);
            monitor.remove_condition(identifier)?;
        }
        Err(error) => println!("monitor APIs skipped: {error}"),
    }
    Ok(())
}
