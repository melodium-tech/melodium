//! Volumes of the worker, shared by the engine and its executors.

use std::path::PathBuf;

/// Gives the directory of `volume` on this worker: listed in `MELODIUM_RUN_VOLUMES`, at the
/// path `MELODIUM_RUN_VOLUME_<volume>` gives.
///
/// Executors write in volumes too: files are reached within this directory through
/// `fs_mel::rooted::RootedFileSystemEngine`, never by joining paths to it.
pub(crate) fn volume_root(volume: &str) -> Result<PathBuf, String> {
    if Ok(true)
        != std::env::var("MELODIUM_RUN_VOLUMES")
            .map(|var| var.split(',').any(|var_volume| var_volume == volume))
    {
        return Err(format!("no volume '{volume}' listed as available"));
    }
    std::env::var_os(format!("MELODIUM_RUN_VOLUME_{volume}"))
        .map(PathBuf::from)
        .ok_or_else(|| format!("no volume '{volume}' available"))
}
