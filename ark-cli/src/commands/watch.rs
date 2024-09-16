use std::path::PathBuf;

use futures::{pin_mut, StreamExt};

use fs_index::{watch_index, WatchEvent};

use crate::{AppError, ResourceId};

#[derive(Clone, Debug, clap::Args)]
#[clap(
    name = "watch",
    about = "Watch the ark managed folder for changes and update the index accordingly"
)]
pub struct Watch {
    #[clap(
        help = "Path to the directory to watch for changes",
        default_value = ".",
        value_parser
    )]
    path: PathBuf,
}

impl Watch {
    pub async fn run(&self) -> Result<(), AppError> {
        let stream = watch_index::<_, ResourceId>(&self.path);
        pin_mut!(stream);

        while let Some(value) = stream.next().await {
            match value {
                WatchEvent::UpdatedOne(path) => {
                    println!("Updated file: {:?}", path);
                }
                WatchEvent::UpdatedAll(update) => {
                    println!("Updated all: {:?}", update);
                }
            }
        }

        Ok(())
    }
}
