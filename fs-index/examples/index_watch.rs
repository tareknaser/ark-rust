use std::{path::Path, thread};

use anyhow::Result;
use log::LevelFilter;

use dev_hash::Blake3;
use fs_index::watch_index;

/// Example demonstrating how to use fs_index to watch a directory for changes
/// in a separate thread. This automatically updates the index when changes are
/// detected.
fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    // Change this to the path of the directory you want to watch
    let root = Path::new("test-assets");

    let thread_handle = thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                if let Err(err) = watch_index::<_, Blake3>(root).await {
                    eprintln!("Error in watching index: {:?}", err);
                }
            });
    });

    thread_handle
        .join()
        .expect("Failed to join thread");

    Ok(())
}
