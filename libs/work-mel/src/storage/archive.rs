//! Archives of directories, as tar compressed with zstd.

use melodium_core::common::executive::RevealedScan;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

/// Compression level of archives, fast enough to archive while reading.
const LEVEL: i32 = 3;

/// Writer searching the values revealed by the engine in what goes through it, and
/// failing once one is found.
struct Scanning<W> {
    inner: W,
    scan: RevealedScan,
    /// Name of the secret whose revealed value was found.
    found: Arc<Mutex<Option<String>>>,
}

impl<W: Write> Write for Scanning<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(name) = self.scan.scan(buf) {
            *self.found.lock().unwrap_or_else(PoisonError::into_inner) = Some(name);
            return Err(std::io::Error::other("revealed value found"));
        }
        // Written whole, so that nothing is searched twice.
        self.inner.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Archives the content of the directory `source` into `file`, giving it back with the size
/// of the archive, ready to be read from its start.
///
/// Links are archived as links, never followed. The archive is refused when `scan` finds a
/// value revealed by the engine in it, names of files included.
pub(crate) fn pack(source: &Path, file: File, scan: RevealedScan) -> Result<(File, u64), String> {
    let found = Arc::new(Mutex::new(None));
    let encoder = zstd::Encoder::new(file, LEVEL).map_err(|err| err.to_string())?;
    let mut builder = tar::Builder::new(Scanning {
        inner: encoder,
        scan,
        found: Arc::clone(&found),
    });
    builder.follow_symlinks(false);

    let packed = builder
        .append_dir_all(".", source)
        .and_then(|()| builder.into_inner())
        .and_then(|scanning| scanning.inner.finish())
        .and_then(|mut file| {
            file.flush()?;
            let size = file.seek(SeekFrom::End(0))?;
            file.seek(SeekFrom::Start(0))?;
            Ok((file, size))
        });
    let found = found.lock().unwrap_or_else(PoisonError::into_inner).take();
    match (packed, found) {
        (_, Some(name)) => Err(format!(
            "it holds the value of secret '{name}' revealed in this run"
        )),
        (Ok(packed), None) => Ok(packed),
        (Err(err), None) => Err(format!("it cannot be archived: {err}")),
    }
}

/// Extracts the archive in `file`, read from its start, into the directory `target`,
/// created if needed, replacing the files it holds.
///
/// Entries reaching out of `target`, through `..`, absolute paths or links, are refused.
pub(crate) fn unpack(mut file: File, target: &Path) -> Result<(), String> {
    file.seek(SeekFrom::Start(0))
        .map_err(|err| err.to_string())?;
    let decoder = zstd::Decoder::new(file).map_err(|err| err.to_string())?;
    let mut archive = tar::Archive::new(decoder);
    archive.set_overwrite(true);
    std::fs::create_dir_all(target).map_err(|err| err.to_string())?;
    archive.unpack(target).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directories_go_through_archives() {
        let base = std::env::temp_dir().join(format!("work-mel-archive-{}", std::process::id()));
        let source = base.join("source");
        let target = base.join("target");
        std::fs::create_dir_all(source.join("deps/nested")).unwrap();
        std::fs::write(source.join("deps/nested/lib.rlib"), b"compiled").unwrap();
        std::fs::write(source.join("top.txt"), b"top").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("top.txt", source.join("link")).unwrap();
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("kept.txt"), b"kept").unwrap();
        std::fs::write(target.join("top.txt"), b"old").unwrap();

        let (file, size) =
            pack(&source, tempfile::tempfile().unwrap(), RevealedScan::none()).unwrap();
        assert!(size > 0);
        unpack(file, &target).unwrap();

        assert_eq!(
            std::fs::read(target.join("deps/nested/lib.rlib")).unwrap(),
            b"compiled"
        );
        assert_eq!(std::fs::read(target.join("top.txt")).unwrap(), b"top");
        assert_eq!(std::fs::read(target.join("kept.txt")).unwrap(), b"kept");
        #[cfg(unix)]
        assert_eq!(
            std::fs::read_link(target.join("link")).unwrap(),
            Path::new("top.txt")
        );

        std::fs::remove_dir_all(&base).unwrap();
    }
}
