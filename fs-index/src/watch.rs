use std::{fs, path::Path};

use anyhow::Result;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use log::info;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

use data_resource::ResourceId;
use fs_storage::ARK_FOLDER;

use crate::ResourceIndex;

/// Watches a given directory for file system changes and automatically updates
/// the resource index.
///
/// This function continuously monitors the specified directory and responds to
/// file system events such as file creation, modification, and deletion. When
/// an event is detected, the function updates the associated resource index and
/// stores the changes.
///
/// The function runs asynchronously, whcih makes it suitable for non-blocking
/// contexts. It uses a recursive watcher to track all changes within the
/// directory tree. Events related to the internal `.ark` folder are ignored to
/// prevent unnecessary updates.
///
/// # Arguments
///
/// * `root_path` - The root directory to be watched. This path is canonicalized
///   to handle symbolic links and relative paths correctly.
pub async fn watch_index<P: AsRef<Path>, Id: ResourceId>(
    root_path: P,
) -> Result<()> {
    log::debug!(
        "Attempting to watch index at root path: {:?}",
        root_path.as_ref()
    );

    let root_path = fs::canonicalize(root_path.as_ref())?;
    let mut index: ResourceIndex<Id> = ResourceIndex::build(&root_path)?;
    index.store()?;

    let (mut watcher, mut rx) = async_watcher()?;
    info!("Watching directory: {:?}", root_path);
    let config = Config::default();
    watcher.configure(config)?;
    watcher.watch(root_path.as_ref(), RecursiveMode::Recursive)?;
    info!("Started watcher with config: \n\t{:?}", config);

    let ark_folder = root_path.join(ARK_FOLDER);
    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                // If the event is a change in .ark folder, ignore it
                if event
                    .paths
                    .iter()
                    .any(|p| p.starts_with(&ark_folder))
                {
                    continue;
                }
                // We only care for:
                // - file modifications
                // - file renames
                // - file creations
                // - file deletions
                match event.kind {
                    notify::EventKind::Modify(
                        notify::event::ModifyKind::Data(_),
                    )
                    | notify::EventKind::Modify(
                        notify::event::ModifyKind::Name(_),
                    )
                    | notify::EventKind::Create(
                        notify::event::CreateKind::File,
                    )
                    | notify::EventKind::Remove(
                        notify::event::RemoveKind::File,
                    ) => {}
                    _ => continue,
                }

                // If the event requires a rescan, update the entire index
                // else, update the index for the specific file
                if event.need_rescan() {
                    info!("Detected rescan event: {:?}", event);
                    index.update_all()?;
                } else {
                    info!("Detected event: {:?}", event);
                    let file = event
                        .paths
                        .first()
                        .expect("Failed to get file path from event");
                    log::debug!("Updating index for file: {:?}", file);

                    log::info!(
                        "\n Current resource index: {}",
                        index
                            .resources()
                            .iter()
                            .map(|x| x.path().to_str().unwrap().to_string())
                            .collect::<Vec<String>>()
                            .join("\n\t")
                    );

                    let relative_path = file.strip_prefix(&root_path)?;
                    log::info!("Relative path: {:?}", relative_path);
                    index.update_one(relative_path)?;
                }

                index.store()?;
                info!("Index updated and stored");
            }
            Err(e) => log::error!("Error in watcher: {:?}", e),
        }
    }

    unreachable!("Watcher stream ended unexpectedly");
}

fn async_watcher(
) -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                if let Err(err) = tx.send(res).await {
                    log::error!("Error sending event: {:?}", err);
                }
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}
