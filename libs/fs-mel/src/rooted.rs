//! File systems rooted at a directory, every path being resolved within it.

use crate::filesystem::*;
use async_trait::async_trait;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use futures::{AsyncReadExt, AsyncWriteExt};
use melodium_macro::check;
use std::io;
use std::path::{Path, PathBuf};

/// File system rooted at a directory.
///
/// Paths are relative to the root, and resolved within it as files are opened, component by
/// component, by the operating system when it can: `..`, absolute paths and links leading out
/// of the root are refused, as are links leading out that are created meanwhile, such as by
/// other containers sharing the directory. Links staying within the root are followed.
#[derive(Debug)]
pub struct RootedFileSystemEngine {
    root: PathBuf,
}

impl RootedFileSystemEngine {
    /// File system rooted at `root`, a directory opened at each access.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Runs `operation` on the root directory, on a thread where blocking is allowed.
    async fn within<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&Dir) -> io::Result<T> + Send + 'static,
    ) -> io::Result<T> {
        let root = self.root.clone();
        blocking::unblock(move || {
            let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
            operation(&dir)
        })
        .await
    }
}

/// Lists the entries of `dir`, as `prefix` followed by their names: files, then directories
/// and their own entries when `recursive`, and links, without following them, when
/// `follow_links`. Errors on entries are given apart.
fn walk(
    dir: &Dir,
    prefix: &Path,
    recursive: bool,
    follow_links: bool,
    found: &mut Vec<PathBuf>,
    problems: &mut Vec<String>,
) -> io::Result<()> {
    for entry in dir.entries()? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                problems.push(err.to_string());
                continue;
            }
        };
        let path = prefix.join(entry.file_name());
        match entry.file_type() {
            Ok(file_type) if file_type.is_dir() => {
                if recursive {
                    found.push(path.clone());
                    if let Err(err) = entry
                        .open_dir()
                        .and_then(|dir| walk(&dir, &path, recursive, follow_links, found, problems))
                    {
                        problems.push(err.to_string());
                    }
                }
            }
            Ok(file_type) if file_type.is_symlink() => {
                if follow_links {
                    found.push(path);
                }
            }
            _ => found.push(path),
        }
    }
    Ok(())
}

#[async_trait]
impl FileSystemEngine for RootedFileSystemEngine {
    async fn read_file(
        &self,
        path: &str,
        data: OutDataCall<'async_trait>,
        reached: OnceTriggerCall<'async_trait>,
        reachedclose: OnceTriggerCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        errors: OutMessageCall<'async_trait>,
    ) {
        let path = PathBuf::from(path);
        let file = self
            .within(move |dir| dir.open(&path).map(|file| file.into_std()))
            .await;
        match file {
            Ok(file) => {
                let mut file = async_fs::File::from(file);
                reached().await;
                reachedclose().await;
                let mut vec = vec![0; 2usize.pow(20)];
                let mut fail = false;
                loop {
                    match file.read(&mut vec).await {
                        Ok(n) if n > 0 => {
                            vec.truncate(n);
                            check!(data(vec.into()).await);
                            vec = vec![0; 2usize.pow(20)];
                        }
                        Ok(_) => {
                            break;
                        }
                        Err(err) => {
                            failed().await;
                            let _ = errors(err.to_string()).await;
                            fail = true;
                            break;
                        }
                    }
                }
                if !fail {
                    completed().await;
                }
            }
            Err(err) => {
                failed().await;
                let _ = errors(err.to_string()).await;
            }
        }
        finished().await;
    }

    async fn write_file(
        &self,
        path: &str,
        append: bool,
        create: bool,
        new: bool,
        data: InDataCall<'async_trait>,
        amount: OutU128Call<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        errors: OutMessageCall<'async_trait>,
    ) {
        let path = PathBuf::from(path);
        let file = self
            .within(move |dir| {
                // A link at the parent is resolved when opening, never replaced by a directory.
                if let Some(parent) = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                {
                    if dir.symlink_metadata(parent).is_err() {
                        dir.create_dir_all(parent)?;
                    }
                }
                dir.open_with(
                    &path,
                    OpenOptions::new()
                        .write(true)
                        .append(append)
                        .create(create)
                        .create_new(new),
                )
                .map(|file| file.into_std())
            })
            .await;
        match file {
            Ok(file) => {
                let mut file = async_fs::File::from(file);
                let mut written_amount = 0u128;
                let mut error = None;
                while let Ok(data) = data().await {
                    match file.write_all(&data).await {
                        Ok(_) => {
                            written_amount += data.len() as u128;
                            let _ = amount(written_amount).await;
                        }
                        Err(err) => {
                            error = Some(err);
                            break;
                        }
                    }
                }
                // Writes run in the background: flushing waits for them, and gives their errors.
                if error.is_none() {
                    error = file.flush().await.err();
                }
                match error {
                    None => completed().await,
                    Some(err) => {
                        failed().await;
                        let _ = errors(err.to_string()).await;
                    }
                }
            }
            Err(err) => {
                failed().await;
                let _ = errors(err.to_string()).await;
            }
        }
        finished().await;
    }

    async fn create_dir(
        &self,
        path: &str,
        recursive: bool,
        success: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
    ) {
        let path = PathBuf::from(path);
        let created = self
            .within(move |dir| {
                if recursive {
                    dir.create_dir_all(&path)
                } else {
                    dir.create_dir(&path)
                }
            })
            .await;
        match created {
            Ok(()) => {
                success().await;
            }
            Err(err) => {
                error(err.to_string()).await;
                failed().await;
            }
        }
    }

    async fn scan_dir(
        &self,
        path: &str,
        recursive: bool,
        follow_links: bool,
        entries: OutMessageCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        errors: OutMessageCall<'async_trait>,
    ) {
        // Entries are given as before, under the root as the engine sees it.
        let base = if path.is_empty() {
            self.root.clone()
        } else {
            self.root.join(path)
        };
        let start = PathBuf::from(if path.is_empty() { "." } else { path });
        let scanned = self
            .within(move |dir| {
                let dir = dir.open_dir(&start)?;
                let mut found = Vec::new();
                let mut problems = Vec::new();
                walk(
                    &dir,
                    Path::new(""),
                    recursive,
                    follow_links,
                    &mut found,
                    &mut problems,
                )?;
                Ok((found, problems))
            })
            .await;

        let success = match scanned {
            Ok((found, problems)) => {
                for entry in found {
                    check!(entries(base.join(entry).to_string_lossy().to_string()).await);
                }
                for problem in &problems {
                    let _ = errors(problem.clone()).await;
                }
                problems.is_empty()
            }
            Err(err) => {
                let _ = errors(err.to_string()).await;
                false
            }
        };
        finished().await;
        if success {
            completed().await;
        } else {
            failed().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::sync::{Arc, Mutex};

    /// Root with `inside/file`, a link `inner` to `inside`, and links `out` and `abs` to a
    /// directory beside the root holding `secret`.
    struct Fixture {
        base: PathBuf,
        root: PathBuf,
        outside: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let base =
                std::env::temp_dir().join(format!("fs-mel-rooted-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&base);
            let root = base.join("root");
            let outside = base.join("outside");
            std::fs::create_dir_all(root.join("inside")).unwrap();
            std::fs::create_dir_all(&outside).unwrap();
            std::fs::write(root.join("inside/file"), b"inside").unwrap();
            std::fs::write(outside.join("secret"), b"outside").unwrap();
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink("inside", root.join("inner")).unwrap();
                std::os::unix::fs::symlink("../outside", root.join("out")).unwrap();
                std::os::unix::fs::symlink(&outside, root.join("abs")).unwrap();
            }
            Self {
                base,
                root,
                outside,
            }
        }

        fn engine(&self) -> RootedFileSystemEngine {
            RootedFileSystemEngine::new(self.root.clone())
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }

    fn trigger<'a>(flag: &Arc<Mutex<bool>>) -> OnceTriggerCall<'a> {
        let flag = Arc::clone(flag);
        Box::new(move || {
            *flag.lock().unwrap() = true;
            Box::pin(async {})
        })
    }

    fn messages<'a>(list: &Arc<Mutex<Vec<String>>>) -> OutMessageCall<'a> {
        let list = Arc::clone(list);
        Box::new(move |message| {
            list.lock().unwrap().push(message);
            Box::pin(async { Ok(()) })
        })
    }

    fn read(engine: &RootedFileSystemEngine, path: &str) -> Result<Vec<u8>, Vec<String>> {
        let content = Arc::new(Mutex::new(Vec::new()));
        let failed = Arc::new(Mutex::new(false));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let unused = Arc::new(Mutex::new(false));
        let into = Arc::clone(&content);
        block_on(engine.read_file(
            path,
            Box::new(move |data| {
                into.lock().unwrap().extend(data);
                Box::pin(async { Ok(()) })
            }),
            trigger(&unused),
            trigger(&unused),
            trigger(&unused),
            trigger(&failed),
            trigger(&unused),
            messages(&errors),
        ));
        if *failed.lock().unwrap() {
            Err(errors.lock().unwrap().clone())
        } else {
            Ok(content.lock().unwrap().clone())
        }
    }

    fn write(
        engine: &RootedFileSystemEngine,
        path: &str,
        content: &[u8],
    ) -> Result<(), Vec<String>> {
        let chunks = Arc::new(Mutex::new(vec![content.to_vec()]));
        let failed = Arc::new(Mutex::new(false));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let unused = Arc::new(Mutex::new(false));
        block_on(engine.write_file(
            path,
            false,
            true,
            false,
            Box::new(move || {
                let chunk = chunks.lock().unwrap().pop();
                Box::pin(async move { chunk.ok_or(()) })
            }),
            Box::new(|_| Box::pin(async { Ok(()) })),
            trigger(&unused),
            trigger(&failed),
            trigger(&unused),
            messages(&errors),
        ));
        if *failed.lock().unwrap() {
            Err(errors.lock().unwrap().clone())
        } else {
            Ok(())
        }
    }

    fn create(engine: &RootedFileSystemEngine, path: &str) -> Result<(), String> {
        let failed = Arc::new(Mutex::new(false));
        let unused = Arc::new(Mutex::new(false));
        let error = Arc::new(Mutex::new(String::new()));
        let into = Arc::clone(&error);
        block_on(engine.create_dir(
            path,
            true,
            trigger(&unused),
            trigger(&failed),
            Box::new(move |message| {
                *into.lock().unwrap() = message;
                Box::pin(async {})
            }),
        ));
        if *failed.lock().unwrap() {
            Err(error.lock().unwrap().clone())
        } else {
            Ok(())
        }
    }

    fn scan(engine: &RootedFileSystemEngine, path: &str) -> Result<Vec<String>, Vec<String>> {
        let found = Arc::new(Mutex::new(Vec::new()));
        let failed = Arc::new(Mutex::new(false));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let unused = Arc::new(Mutex::new(false));
        block_on(engine.scan_dir(
            path,
            true,
            true,
            messages(&found),
            trigger(&unused),
            trigger(&failed),
            trigger(&unused),
            messages(&errors),
        ));
        if *failed.lock().unwrap() {
            Err(errors.lock().unwrap().clone())
        } else {
            let mut found = found.lock().unwrap().clone();
            found.sort();
            Ok(found)
        }
    }

    #[test]
    fn files_within_the_root_are_reached() {
        let fixture = Fixture::new("within");
        let engine = fixture.engine();

        assert_eq!(read(&engine, "inside/file").unwrap(), b"inside");
        assert_eq!(read(&engine, "inside/../inside/file").unwrap(), b"inside");
        write(&engine, "new/deep/file", b"written").unwrap();
        assert_eq!(
            std::fs::read(fixture.root.join("new/deep/file")).unwrap(),
            b"written"
        );
        create(&engine, "made/dir").unwrap();
        assert!(fixture.root.join("made/dir").is_dir());
        #[cfg(unix)]
        assert_eq!(read(&engine, "inner/file").unwrap(), b"inside");

        let root = fixture.root.to_string_lossy().to_string();
        let found = scan(&engine, "").unwrap();
        for entry in ["inside", "inside/file", "made", "made/dir", "new/deep/file"] {
            assert!(
                found.contains(&format!("{root}/{entry}")),
                "{entry} in {found:?}"
            );
        }
    }

    #[test]
    fn paths_leaving_the_root_are_refused() {
        let fixture = Fixture::new("leaving");
        let engine = fixture.engine();
        let secret = fixture.outside.join("secret").to_string_lossy().to_string();

        let mut leaving = vec![
            "../outside/secret",
            "inside/../../outside/secret",
            secret.as_str(),
        ];
        #[cfg(unix)]
        leaving.extend(["out/secret", "abs/secret"]);
        for path in leaving {
            assert!(read(&engine, path).is_err(), "read {path}");
            assert!(
                write(&engine, &path.replace("secret", "written"), b"x").is_err(),
                "write {path}"
            );
            assert!(
                create(&engine, &path.replace("secret", "made")).is_err(),
                "create {path}"
            );
            assert!(
                scan(&engine, &path.replace("/secret", "")).is_err(),
                "scan {path}"
            );
        }
        // Directories to create on the way are never created outside either.
        for path in [
            "../made/deeper/file",
            "../outside/made/file",
            "inside/../../made/file",
        ] {
            assert!(write(&engine, path, b"x").is_err(), "write {path}");
            assert!(
                create(&engine, &path.replace("/file", "")).is_err(),
                "create {path}"
            );
        }
        assert_eq!(
            std::fs::read_dir(&fixture.outside).unwrap().count(),
            1,
            "nothing was written outside"
        );
        assert!(
            !fixture.base.join("made").exists(),
            "nothing was created beside the root"
        );
        assert_eq!(
            read(&engine, "../outside/secret").unwrap_err(),
            vec!["a path led outside of the filesystem".to_string()]
        );
    }

    #[cfg(unix)]
    #[test]
    fn links_made_meanwhile_are_refused() {
        let fixture = Fixture::new("meanwhile");
        let engine = fixture.engine();
        assert_eq!(read(&engine, "inside/file").unwrap(), b"inside");

        // Another container sharing the root turns a directory into a link leading out.
        std::fs::remove_dir_all(fixture.root.join("inside")).unwrap();
        std::fs::create_dir_all(fixture.outside.join("inside")).unwrap();
        std::fs::write(fixture.outside.join("inside/file"), b"outside").unwrap();
        std::os::unix::fs::symlink(fixture.outside.join("inside"), fixture.root.join("inside"))
            .unwrap();

        assert!(read(&engine, "inside/file").is_err());
        assert!(write(&engine, "inside/file", b"overwritten").is_err());
        assert_eq!(
            std::fs::read(fixture.outside.join("inside/file")).unwrap(),
            b"outside"
        );
    }
}
