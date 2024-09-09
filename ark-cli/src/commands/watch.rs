use std::path::PathBuf;

use fs_index::watch_index;

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
        watch_index::<_, ResourceId>(&self.path)
            .await
            .map_err(|err| AppError::IndexError(err.to_string()))
    }
}
