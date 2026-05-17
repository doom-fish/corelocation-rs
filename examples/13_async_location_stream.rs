//! Example 13: Async location stream
//!
//! Demonstrates [`LocationManagerStream`]: subscribes to location updates and
//! authorization-change events using `pollster::block_on`.
//!
//! On a headless CI machine the `CLLocationManager` will not produce live GPS
//! fixes, so the example starts the stream, checks the initial authorization
//! state via `try_next`, and exits gracefully without blocking forever.
//!
//! ```
//! cargo run --example 13_async_location_stream --features async
//! ```

use std::time::Duration;

use corelocation::async_api::{LocationManagerEvent, LocationManagerStream};

fn main() {
    println!("=== async location stream example ===");

    let stream = match LocationManagerStream::new(32) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not create LocationManagerStream: {e}");
            std::process::exit(0);
        }
    };

    println!("Stream created: {stream:?}");

    // Start location updates so the delegate fires at least an error or auth event.
    stream.start_updating_location();
    stream.start_updating_heading();

    // Give CoreLocation ~200 ms to fire the initial authorization callback.
    std::thread::sleep(Duration::from_millis(200));

    // Drain whatever arrived without blocking (headless-CI-safe).
    let mut count = 0usize;
    while let Some(event) = stream.try_next() {
        count += 1;
        match &event {
            LocationManagerEvent::DidUpdateLocations(locs) => {
                println!("  [DidUpdateLocations] {} fix(es)", locs.len());
                for loc in locs {
                    println!("    lat={:.4} lon={:.4}", loc.coordinate.latitude, loc.coordinate.longitude);
                }
            }
            LocationManagerEvent::DidFailWithError(err) => {
                println!("  [DidFailWithError] {} (code {})", err.message, err.code);
            }
            LocationManagerEvent::DidChangeAuthorization(auth) => {
                println!("  [DidChangeAuthorization] status={:?}", auth.status);
            }
            LocationManagerEvent::DidUpdateHeading(h) => {
                println!("  [DidUpdateHeading] magnetic={:.1}°", h.magnetic_heading);
            }
            LocationManagerEvent::DidEnterRegion(r) => {
                println!("  [DidEnterRegion] {r:?}");
            }
            LocationManagerEvent::DidExitRegion(r) => {
                println!("  [DidExitRegion] {r:?}");
            }
            LocationManagerEvent::DidVisit(v) => {
                println!("  [DidVisit] {v:?}");
            }
            _ => {}
        }
    }

    println!("Drained {count} event(s). Stream: {stream:?}");

    // Also show pollster::block_on usage with a short-circuit timeout pattern.
    let received = pollster::block_on(async {
        // Use try_next in a tiny poll loop rather than parking forever.
        for _ in 0..5 {
            if let Some(ev) = stream.try_next() {
                return Some(ev);
            }
            // Yield to give other async work a chance (no-op in pollster).
            std::hint::black_box(());
        }
        None
    });
    if let Some(ev) = received {
        println!("block_on got: {ev:?}");
    } else {
        println!("block_on: no buffered events (expected on headless machine)");
    }

    println!("Dropping stream...");
    drop(stream);
    println!("Done.");
}
