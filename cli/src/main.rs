use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use clap::Parser;
use common::{
    sidecar::{open_or_create_sidecar, open_sidecar, Sidecar},
    VIDEO_EXTENSIONS,
};

#[derive(Parser)]
pub enum Cli {
    /// Mark a video file as watched
    Watched {
        /// File to mark. Must be a video file.
        file: PathBuf,
    },
    /// Mark a video file as unwatched
    Unwatched {
        /// File to mark. Must be a video file.
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args {
        Cli::Watched { file } => {
            let file_name = get_file_name(&file)?;
            let sidecar = open_or_create_sidecar(&file)?;
            let mut sidecar = Sidecar::new(sidecar)?;
            sidecar.add(&file_name)?;
        }
        Cli::Unwatched { file } => {
            let file_name = get_file_name(&file)?;
            let Some(sidecar) = open_sidecar(&file) else {
                return Ok(());
            };

            let mut sidecar = Sidecar::new(sidecar?)?;
            sidecar.remove(&file_name)?;
        }
    }

    Ok(())
}

fn get_file_name(file: &Path) -> Result<String> {
    let ext = file
        .extension()
        .context("Provided path has no extension")?
        .to_string_lossy();

    if !VIDEO_EXTENSIONS.contains(&ext.borrow()) {
        bail!("Provided file is not a video file");
    }

    let file_name = file
        .file_name()
        .context("Provided path is not a file")?
        .to_string_lossy();

    Ok(file_name.into_owned())
}
