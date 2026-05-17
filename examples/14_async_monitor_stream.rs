//! Example 14: Async `CLMonitor` stream (macOS 14+)
//!
//! Demonstrates [`MonitorStream`]: creates a `CLMonitor` stream, registers a
//! circular geographic condition, and drains events non-blockingly.
//!
//! On macOS < 14 the stream creation returns an error and the example exits 0.
//! On headless machines no actual geo-fence events fire; the example exercises
//! the subscribe → `add_condition` → `try_next` → drop lifecycle.
//!
//! ```
//! cargo run --example 14_async_monitor_stream --features async
//! ```

use corelocation::async_api::{MonitorStream, MonitorStreamEvent};
use corelocation::location::Coordinate;
use corelocation::monitor::CircularGeographicCondition;

fn main() {
    println!("=== async monitor stream example ===");

    let stream = match MonitorStream::new("examplemonitor", 16) {
        Ok(s) => s,
        Err(e) => {
            // macOS < 14 or permission denied — not a test failure.
            println!("MonitorStream::new failed (likely macOS < 14): {e}");
            println!("Exiting 0 (not a failure).");
            return;
        }
    };

    println!("MonitorStream created: {stream:?}");

    // Apple Park, Cupertino
    let coord = Coordinate { latitude: 37.3318, longitude: -122.0312 };
    let condition = match CircularGeographicCondition::new(coord, 150.0) {
        Ok(c) => c,
        Err(e) => {
            println!("Could not create condition: {e}");
            return;
        }
    };

    match stream.add_condition(&condition, "applepark") {
        Ok(()) => println!("Added condition 'apple-park'"),
        Err(e) => println!("add_condition error (non-fatal): {e}"),
    }

    // Give the monitor a moment to produce initial state events.
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut count = 0usize;
    while let Some(event) = stream.try_next() {
        count += 1;
        match &event {
            MonitorStreamEvent::DidChange(ev) => {
                println!(
                    "  [DidChange] id='{}' state={:?}",
                    ev.identifier, ev.state
                );
            }
            MonitorStreamEvent::Error(err) => {
                println!("  [Error] {} (code {})", err.message, err.code);
            }
            _ => {}
        }
    }

    println!("Drained {count} event(s). Stream: {stream:?}");

    // Remove the condition before dropping.
    if let Err(e) = stream.remove_condition("applepark") {
        println!("remove_condition error (non-fatal): {e}");
    }

    println!("Dropping stream...");
    drop(stream);
    println!("Done.");
}
