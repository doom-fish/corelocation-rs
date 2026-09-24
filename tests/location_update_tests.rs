use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod common;

use corelocation::prelude::*;

#[test]
fn location_updater_creation_and_invalidation_smoke() -> Result<(), Box<dyn std::error::Error>> {
    if LocationUpdater::is_supported() {
        let updater = LocationUpdater::new()?;
        updater.invalidate();
    }
    Ok(())
}

#[derive(Default)]
struct Counters {
    updates: AtomicUsize,
    invalidations: AtomicUsize,
}

impl Counters {
    fn updates(&self) -> usize {
        self.updates.load(Ordering::SeqCst)
    }

    fn invalidations(&self) -> usize {
        self.invalidations.load(Ordering::SeqCst)
    }
}

fn counting_updater(counters: &Arc<Counters>) -> Result<LocationUpdater, CoreLocationError> {
    let updates = Arc::clone(counters);
    let invalidations = Arc::clone(counters);
    LocationUpdater::with_callbacks(
        LocationUpdateCallbacks::new()
            .on_update(move |_| {
                updates.updates.fetch_add(1, Ordering::SeqCst);
            })
            .on_invalidate(move || {
                invalidations.invalidations.fetch_add(1, Ordering::SeqCst);
            }),
    )
}

fn wait_for(limit: Duration, condition: impl Fn() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < limit {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    condition()
}

#[test]
fn dropping_a_resumed_updater_stops_its_task_and_frees_the_delegate(
) -> Result<(), Box<dyn std::error::Error>> {
    if !common::live_tests_enabled() || !LocationUpdater::is_supported() {
        return Ok(());
    }
    let counters = Arc::new(Counters::default());
    let updater = counting_updater(&counters)?;
    updater.resume();
    let _ = wait_for(Duration::from_secs(2), || counters.updates() > 0);

    drop(updater);
    let delivered = counters.updates();
    assert_eq!(Arc::strong_count(&counters), 1);
    thread::sleep(Duration::from_millis(200));
    assert_eq!(counters.updates(), delivered);
    assert_eq!(counters.invalidations(), 0);
    Ok(())
}

#[test]
fn pausing_does_not_report_invalidation_and_resume_starts_a_new_run(
) -> Result<(), Box<dyn std::error::Error>> {
    if !common::live_tests_enabled() || !LocationUpdater::is_supported() {
        return Ok(());
    }
    let counters = Arc::new(Counters::default());
    let updater = counting_updater(&counters)?;

    updater.resume();
    let delivers_initial_update = wait_for(Duration::from_secs(2), || counters.updates() > 0);
    updater.pause();
    let after_first_pause = counters.updates();

    updater.resume();
    if delivers_initial_update {
        let resumed = wait_for(Duration::from_secs(2), || {
            counters.updates() > after_first_pause
        });
        assert!(
            resumed,
            "resume after pause must start a new live-update run"
        );
    }
    updater.pause();
    let after_second_pause = counters.updates();
    thread::sleep(Duration::from_millis(200));

    assert_eq!(counters.updates(), after_second_pause);
    assert_eq!(counters.invalidations(), 0);
    updater.invalidate();
    updater.resume();
    thread::sleep(Duration::from_millis(100));
    assert_eq!(counters.updates(), after_second_pause);

    drop(updater);
    assert_eq!(Arc::strong_count(&counters), 1);
    Ok(())
}
