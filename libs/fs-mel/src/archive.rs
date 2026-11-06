//! Tar archives of directories of file systems, made and extracted while their data flows.
//!
//! Archives hold files, directories and links, links being kept as links and never followed.
//! They are deterministic: entries are sorted by name, with their mode and modification time,
//! owned by 0 without user or group names, so that the same files give the same bytes.
//! Extraction never writes out of its target.

use crate::filesystem::*;
use melodium_core::*;
use melodium_macro::mel_treatment;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tar::{EntryType, Header};

/// Size of tar blocks.
const BLOCK: usize = 512;
/// Longest name a tar header holds, longer ones going in a long name entry before it.
const NAME_LENGTH: usize = 100;
/// Name of long name entries.
const LONG_LINK: &[u8] = b"././@LongLink";
/// Size of the chunks of files given through the archive.
const CHUNK: usize = 1 << 20;

/// Archive a directory.
///
/// The directory `path` of `filesystem` is archived as tar, given through `data` while its
/// files are read, one at a time: nothing is written elsewhere, and memory does not grow with
/// the directory.
///
/// Files, directories and links are archived, links as links, never followed, and paths are
/// relative to `path`. Entries are sorted by name, with their mode and modification time (to
/// the second), owned by 0 without user or group names: the same files give the same bytes.
/// Other entries (devices, sockets, pipes) are left out, each named in `errors`.
///
/// Once the archive is complete, `completed` is emitted. If `path` is not a directory, or an
/// entry cannot be read or changes while archived, `failed` is emitted and `errors` contains
/// the reason: the data given before is not a complete archive. `finished` is emitted in every
/// case.
///
/// ```mermaid
/// graph LR
///     T("tar()")
///     F["〈🟨〉"] -->|filesystem| T
///     P["〈🟨〉"] -->|path| T
///
///     T -->|data| O["… 🟦 🟦 🟦"]
///     T -->|completed| C["〈🟩〉"]
///
///     style F fill:#ffffff,stroke:#ffffff
///     style P fill:#ffffff,stroke:#ffffff
///     style O fill:#ffffff,stroke:#ffffff
///     style C fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input filesystem Block<FileSystem>
    input path Block<string>
    output data Stream<byte>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output errors Stream<string>
)]
pub async fn tar() {
    if let (Ok(filesystem), Ok(path)) = (
        filesystem.recv_one_as::<Arc<FileSystem>>().await,
        path.recv_one_as::<string>().await,
    ) {
        let mut left_out = Vec::new();
        let result = archive(
            &*filesystem.filesystem,
            &path,
            &mut |bytes| {
                let data = &data;
                async move {
                    data.send_many(TransmissionValue::Byte(bytes.into()))
                        .await
                        .is_ok()
                }
            },
            &mut left_out,
        )
        .await;
        data.close().await;
        for message in left_out {
            let _ = errors.send_one_as(message).await;
        }
        match result {
            Ok(true) => {
                let _ = completed.send_one_as(()).await;
            }
            Ok(false) => {}
            Err(reason) => {
                let _ = failed.send_one_as(()).await;
                let _ = errors.send_one_as(reason).await;
            }
        }
    }
    let _ = finished.send_one_as(()).await;
}

/// Extract an archive.
///
/// The tar archive received through `data` is extracted to the directory `path` of
/// `filesystem`, its entries being written while they come. `path` must not exist: it is
/// created, so that an archive never mixes with existing content.
///
/// Files, directories and symbolic links are extracted, with their mode and modification time,
/// those of directories once their content is written. Links are made as written, and never
/// followed. An entry with `..` or an absolute path, or going through a link of the archive,
/// fails the extraction, as do hard links and other kinds of entries: nothing is ever written
/// out of `path`.
///
/// Once the archive is extracted, `completed` is emitted. If it fails, `failed` is emitted and
/// `errors` contains the reason, what was extracted being left as is: extract into a directory
/// of your own, and remove it on failure. `finished` is emitted in every case.
///
/// ```mermaid
/// graph LR
///     T("untar()")
///     F["〈🟨〉"] -->|filesystem| T
///     P["〈🟨〉"] -->|path| T
///     B["… 🟦 🟦 🟦"] -->|data| T
///
///     T -->|completed| C["〈🟩〉"]
///     T -->|failed| E["〈🟥〉"]
///
///     style F fill:#ffffff,stroke:#ffffff
///     style P fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
///     style C fill:#ffffff,stroke:#ffffff
///     style E fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input filesystem Block<FileSystem>
    input path Block<string>
    input data Stream<byte>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output errors Stream<string>
)]
pub async fn untar() {
    if let (Ok(filesystem), Ok(path)) = (
        filesystem.recv_one_as::<Arc<FileSystem>>().await,
        path.recv_one_as::<string>().await,
    ) {
        let result = extract(&*filesystem.filesystem, &path, &mut || async {
            data.recv_many_as::<u8>().await.ok()
        })
        .await;
        data.close();
        match result {
            Ok(()) => {
                let _ = completed.send_one_as(()).await;
            }
            Err(reason) => {
                let _ = failed.send_one_as(()).await;
                let _ = errors.send_one_as(reason).await;
            }
        }
    }
    let _ = finished.send_one_as(()).await;
}

/// Joins `relative` to `root`, as paths of file systems are given.
fn joined(root: &str, relative: &Path) -> String {
    if root.is_empty() {
        relative.to_string_lossy().to_string()
    } else {
        Path::new(root).join(relative).to_string_lossy().to_string()
    }
}

/// Seconds since the epoch of `time`, 0 before it.
fn seconds(time: Option<SystemTime>) -> u64 {
    time.and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Bytes completing `size` to a whole number of blocks.
fn padding(size: u64) -> usize {
    (BLOCK - (size % BLOCK as u64) as usize) % BLOCK
}

/// Copies `bytes` in `field`, cut to its length.
fn set_bytes(field: &mut [u8], bytes: &[u8]) {
    let length = bytes.len().min(field.len());
    field.fill(0);
    field[..length].copy_from_slice(&bytes[..length]);
}

/// Long name entry of `kind` holding `name`.
fn long_entry(kind: EntryType, name: &[u8]) -> Vec<u8> {
    let mut header = Header::new_gnu();
    set_bytes(&mut header.as_old_mut().name, LONG_LINK);
    header.set_entry_type(kind);
    header.set_size(name.len() as u64 + 1);
    header.set_mode(0o644);
    header.set_mtime(0);
    header.set_uid(0);
    header.set_gid(0);
    header.set_cksum();
    let mut entry = header.as_bytes().to_vec();
    entry.extend_from_slice(name);
    entry.push(0);
    entry.extend(std::iter::repeat(0).take(padding(name.len() as u64 + 1)));
    entry
}

/// Headers of an entry, preceded by long name entries when its name or link need them.
fn headers(
    name: &[u8],
    kind: EntryType,
    size: u64,
    mode: u32,
    modified: u64,
    link: Option<&[u8]>,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    if name.len() > NAME_LENGTH {
        bytes.extend(long_entry(EntryType::GNULongName, name));
    }
    if let Some(link) = link.filter(|link| link.len() > NAME_LENGTH) {
        bytes.extend(long_entry(EntryType::GNULongLink, link));
    }
    let mut header = Header::new_gnu();
    set_bytes(&mut header.as_old_mut().name, name);
    if let Some(link) = link {
        set_bytes(&mut header.as_old_mut().linkname, link);
    }
    header.set_entry_type(kind);
    header.set_size(size);
    header.set_mode(mode & 0o7777);
    header.set_mtime(modified);
    header.set_uid(0);
    header.set_gid(0);
    header.set_cksum();
    bytes.extend_from_slice(header.as_bytes());
    bytes
}

/// Archives the directory `root` of `filesystem`, giving the archive through `give`, which
/// tells whether the bytes were taken. Entries left out are named in `left_out`.
///
/// Gives `Ok(false)` when bytes are not taken anymore.
pub(crate) async fn archive<F, Fut>(
    filesystem: &dyn FileSystemEngine,
    root: &str,
    give: &mut F,
    left_out: &mut Vec<String>,
) -> Result<bool, String>
where
    F: FnMut(Vec<u8>) -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    if filesystem.metadata(root).await?.kind != EntryKind::Directory {
        return Err(format!("'{root}' is not a directory"));
    }

    // Entries left to archive, the next one last, for a depth-first walk sorted by name.
    let mut pending = children(filesystem, root, Path::new("")).await?;
    while let Some(relative) = pending.pop() {
        let full = joined(root, &relative);
        let name = relative.to_string_lossy().as_bytes().to_vec();
        let metadata = filesystem.metadata(&full).await?;
        let mode = metadata.mode.unwrap_or(0o644);
        let modified = seconds(metadata.modified);
        match metadata.kind {
            EntryKind::Directory => {
                let mut name = name;
                name.push(b'/');
                if !give(headers(
                    &name,
                    EntryType::Directory,
                    0,
                    mode,
                    modified,
                    None,
                ))
                .await
                {
                    return Ok(false);
                }
                pending.extend(children(filesystem, &full, &relative).await?);
            }
            EntryKind::File => {
                if !give(headers(
                    &name,
                    EntryType::Regular,
                    metadata.size,
                    mode,
                    modified,
                    None,
                ))
                .await
                {
                    return Ok(false);
                }
                if !content(filesystem, &full, metadata.size, give).await? {
                    return Ok(false);
                }
            }
            EntryKind::Link => {
                let target = filesystem.read_link(&full).await?;
                if !give(headers(
                    &name,
                    EntryType::Symlink,
                    0,
                    mode,
                    modified,
                    Some(target.as_bytes()),
                ))
                .await
                {
                    return Ok(false);
                }
            }
            EntryKind::Other => left_out.push(format!(
                "'{}' is neither a file, a directory nor a link, and is left out",
                relative.to_string_lossy()
            )),
        }
    }
    Ok(give(vec![0; 2 * BLOCK]).await)
}

/// Entries of the directory `full`, as paths relative to the archive, sorted the last first.
async fn children(
    filesystem: &dyn FileSystemEngine,
    full: &str,
    relative: &Path,
) -> Result<Vec<PathBuf>, String> {
    let mut names = filesystem.read_dir(full).await?;
    names.sort_unstable_by(|a, b| b.cmp(a));
    Ok(names.into_iter().map(|name| relative.join(name)).collect())
}

/// Gives the `size` bytes of the file `full`, and the padding following them.
async fn content<F, Fut>(
    filesystem: &dyn FileSystemEngine,
    full: &str,
    size: u64,
    give: &mut F,
) -> Result<bool, String>
where
    F: FnMut(Vec<u8>) -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let (sender, receiver) = async_channel::bounded::<VecDeque<u8>>(1);
    let problems = Arc::new(Mutex::new(Vec::new()));
    let reported = Arc::clone(&problems);

    let reading = filesystem.read_file(
        full,
        Box::new(move |chunk| {
            let sender = sender.clone();
            Box::pin(async move { sender.send(chunk).await.map_err(|_| ()) })
        }),
        Box::new(|| Box::pin(async {})),
        Box::new(|| Box::pin(async {})),
        Box::new(|| Box::pin(async {})),
        Box::new(|| Box::pin(async {})),
        Box::new(|| Box::pin(async {})),
        Box::new(move |message| {
            reported.lock().unwrap().push(message);
            Box::pin(async { Ok(()) })
        }),
    );
    let giving = async {
        let mut read: u64 = 0;
        let mut taken = true;
        while let Ok(chunk) = receiver.recv().await {
            let previous = read;
            read += chunk.len() as u64;
            if taken && previous < size {
                let mut chunk: Vec<u8> = chunk.into();
                chunk.truncate((size - previous) as usize);
                taken = give(chunk).await;
            }
        }
        (read, taken)
    };
    let ((), (read, taken)) = futures::join!(reading, giving);

    let problems = core::mem::take(&mut *problems.lock().unwrap());
    if !problems.is_empty() {
        return Err(format!("'{full}' cannot be read: {}", problems.join(", ")));
    }
    if !taken {
        return Ok(false);
    }
    if read != size {
        return Err(format!("'{full}' changed while it was archived"));
    }
    Ok(give(vec![0; padding(size)]).await)
}

/// Data of an archive being extracted.
struct Input<N> {
    next: N,
    buffer: VecDeque<u8>,
}

impl<N, Fut> Input<N>
where
    N: FnMut() -> Fut,
    Fut: std::future::Future<Output = Option<Vec<u8>>>,
{
    /// Makes `length` bytes available, telling whether the data had them.
    async fn fill(&mut self, length: usize) -> bool {
        while self.buffer.len() < length {
            match (self.next)().await {
                Some(bytes) => self.buffer.extend(bytes),
                None => return false,
            }
        }
        true
    }

    /// Takes `length` bytes, failing when the data ends before.
    async fn take(&mut self, length: usize) -> Result<Vec<u8>, String> {
        if self.fill(length).await {
            Ok(self.buffer.drain(..length).collect())
        } else {
            Err("the archive ends within an entry".to_string())
        }
    }

    /// Takes from 1 to `most` bytes, failing when the data ends before.
    async fn some(&mut self, most: usize) -> Result<Vec<u8>, String> {
        if self.fill(1).await {
            let length = most.min(self.buffer.len());
            Ok(self.buffer.drain(..length).collect())
        } else {
            Err("the archive ends within an entry".to_string())
        }
    }
}

/// Values of a PAX extended header that extraction uses.
#[derive(Default)]
struct Pax {
    path: Option<Vec<u8>>,
    link: Option<Vec<u8>>,
    modified: Option<u64>,
}

impl Pax {
    /// Reads the records `<length> <key>=<value>\n` of a PAX extended header.
    fn read(records: &[u8]) -> Result<Self, String> {
        let invalid = || "a PAX header of the archive is not valid".to_string();
        let mut pax = Self::default();
        let mut rest = records;
        while !rest.is_empty() && rest[0] != 0 {
            let space = rest.iter().position(|b| *b == b' ').ok_or_else(invalid)?;
            let length: usize = std::str::from_utf8(&rest[..space])
                .ok()
                .and_then(|length| length.parse().ok())
                .filter(|length| *length > space + 1 && *length <= rest.len())
                .ok_or_else(invalid)?;
            let record = &rest[space + 1..length];
            let record = record.strip_suffix(b"\n").ok_or_else(invalid)?;
            let equal = record.iter().position(|b| *b == b'=').ok_or_else(invalid)?;
            let (key, value) = (&record[..equal], &record[equal + 1..]);
            match key {
                b"path" => pax.path = Some(value.to_vec()),
                b"linkpath" => pax.link = Some(value.to_vec()),
                b"mtime" => {
                    pax.modified = std::str::from_utf8(value)
                        .ok()
                        .and_then(|time| time.split('.').next())
                        .and_then(|seconds| seconds.parse().ok())
                }
                _ => {}
            }
            rest = &rest[length..];
        }
        Ok(pax)
    }
}

/// Gives the relative path of an entry named `name`, refusing names leading out of the
/// archive, and none for names designating the archive itself.
fn entry_path(name: &[u8]) -> Result<Option<PathBuf>, String> {
    let shown = String::from_utf8_lossy(name).to_string();
    let name = std::str::from_utf8(name)
        .map_err(|_| format!("'{shown}' is not a valid name in the archive"))?;
    let mut path = PathBuf::new();
    for component in Path::new(name).components() {
        match component {
            Component::Normal(part) => path.push(part),
            Component::CurDir => {}
            _ => return Err(format!("'{shown}' leads out of the archive")),
        }
    }
    Ok((!path.as_os_str().is_empty()).then_some(path))
}

/// Extracts the archive given by `next` to `root` of `filesystem`, which must not exist.
pub(crate) async fn extract<N, Fut>(
    filesystem: &dyn FileSystemEngine,
    root: &str,
    next: &mut N,
) -> Result<(), String>
where
    N: FnMut() -> Fut,
    Fut: std::future::Future<Output = Option<Vec<u8>>>,
{
    if filesystem.metadata(root).await.is_ok() {
        return Err(format!("'{root}' already exists"));
    }
    create_dir(filesystem, root).await?;

    let mut input = Input {
        next,
        buffer: VecDeque::new(),
    };
    let mut long_name: Option<Vec<u8>> = None;
    let mut long_link: Option<Vec<u8>> = None;
    let mut pax = Pax::default();
    let mut links: HashSet<PathBuf> = HashSet::new();
    let mut directories: HashMap<PathBuf, (u32, u64)> = HashMap::new();

    loop {
        if !input.fill(BLOCK).await {
            return Err("the archive ends before its end".to_string());
        }
        let block: Vec<u8> = input.buffer.drain(..BLOCK).collect();
        if block.iter().all(|byte| *byte == 0) {
            // End of the archive, what follows is ignored.
            break;
        }
        let header = Header::from_byte_slice(&block);
        if header.cksum().ok() != Some(checksum(&block)) {
            return Err("the data is not a valid tar archive".to_string());
        }
        let size = header
            .entry_size()
            .map_err(|_| "an entry of the archive has no valid size".to_string())?;
        let kind = header.entry_type();

        match kind {
            EntryType::GNULongName
            | EntryType::GNULongLink
            | EntryType::XHeader
            | EntryType::XGlobalHeader => {
                let length = usize::try_from(size)
                    .ok()
                    .filter(|length| *length <= 1 << 20)
                    .ok_or_else(|| "an extended header of the archive is too long".to_string())?;
                let mut value = input.take(length).await?;
                input.take(padding(size)).await?;
                match kind {
                    EntryType::GNULongName | EntryType::GNULongLink => {
                        while value.last() == Some(&0) {
                            value.pop();
                        }
                        if kind == EntryType::GNULongName {
                            long_name = Some(value);
                        } else {
                            long_link = Some(value);
                        }
                    }
                    EntryType::XHeader => pax = Pax::read(&value)?,
                    _ => {}
                }
                continue;
            }
            _ => {}
        }

        let entry_pax = core::mem::take(&mut pax);
        let name = long_name
            .take()
            .or(entry_pax.path)
            .unwrap_or_else(|| header.path_bytes().to_vec());
        let link = long_link
            .take()
            .or(entry_pax.link)
            .or_else(|| header.link_name_bytes().map(|link| link.to_vec()));
        let mode = header.mode().unwrap_or(0o644) & 0o7777;
        let modified = entry_pax
            .modified
            .or_else(|| header.mtime().ok())
            .unwrap_or(0);
        let shown = String::from_utf8_lossy(&name).to_string();

        let Some(relative) = entry_path(&name)? else {
            input.take(size as usize).await?;
            input.take(padding(size)).await?;
            continue;
        };
        if relative
            .ancestors()
            .skip(1)
            .any(|ancestor| links.contains(ancestor))
        {
            return Err(format!("'{shown}' goes through a link of the archive"));
        }
        let full = joined(root, &relative);

        match kind {
            EntryType::Directory => {
                create_dir(filesystem, &full).await?;
                directories.insert(relative, (mode, modified));
            }
            EntryType::Regular | EntryType::Continuous => {
                write(filesystem, &full, size, &mut input).await?;
                input.take(padding(size)).await?;
                filesystem
                    .set_metadata(&full, Some(mode), Some(time(modified)))
                    .await?;
            }
            EntryType::Symlink => {
                let target = link
                    .ok_or_else(|| format!("'{shown}' is a link without target"))
                    .and_then(|target| {
                        String::from_utf8(target)
                            .map_err(|_| format!("'{shown}' has a link target that is not valid"))
                    })?;
                filesystem.create_link(&target, &full).await?;
                links.insert(relative);
            }
            EntryType::Link => {
                return Err(format!("'{shown}' is a hard link, which is not extracted"))
            }
            other => {
                return Err(format!(
                    "'{shown}' is of a kind that is not extracted ({:?})",
                    other
                ))
            }
        }
    }

    // Directories get their times and modes once their content is written, deepest first.
    let mut directories: Vec<_> = directories.into_iter().collect();
    directories.sort_unstable_by(|(a, _), (b, _)| b.cmp(a));
    for (relative, (mode, modified)) in directories {
        filesystem
            .set_metadata(&joined(root, &relative), Some(mode), Some(time(modified)))
            .await?;
    }
    Ok(())
}

/// Time `seconds` after the epoch.
fn time(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

/// Checksum of a tar header, its checksum field counted as spaces.
fn checksum(block: &[u8]) -> u32 {
    block
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            if (148..156).contains(&index) {
                b' ' as u32
            } else {
                *byte as u32
            }
        })
        .sum()
}

/// Creates the directory `path` and its missing parents.
async fn create_dir(filesystem: &dyn FileSystemEngine, path: &str) -> Result<(), String> {
    let problem = Arc::new(Mutex::new(None));
    let reported = Arc::clone(&problem);
    filesystem
        .create_dir(
            path,
            true,
            Box::new(|| Box::pin(async {})),
            Box::new(|| Box::pin(async {})),
            Box::new(move |message| {
                *reported.lock().unwrap() = Some(message);
                Box::pin(async {})
            }),
        )
        .await;
    let problem = problem.lock().unwrap().take();
    match problem {
        Some(message) => Err(format!("'{path}' cannot be created: {message}")),
        None => Ok(()),
    }
}

/// Writes the next `size` bytes of `input` to the new file `path`.
async fn write<N, Fut>(
    filesystem: &dyn FileSystemEngine,
    path: &str,
    size: u64,
    input: &mut Input<N>,
) -> Result<(), String>
where
    N: FnMut() -> Fut,
    Fut: std::future::Future<Output = Option<Vec<u8>>>,
{
    let (sender, receiver) = async_channel::bounded::<Vec<u8>>(1);
    let problems = Arc::new(Mutex::new(Vec::new()));
    let reported = Arc::clone(&problems);

    let writing = filesystem.write_file(
        path,
        false,
        true,
        true,
        Box::new(move || {
            let receiver = receiver.clone();
            Box::pin(async move { receiver.recv().await.map_err(|_| ()) })
        }),
        Box::new(|_| Box::pin(async { Ok(()) })),
        Box::new(|| Box::pin(async {})),
        Box::new(|| Box::pin(async {})),
        Box::new(|| Box::pin(async {})),
        Box::new(move |message| {
            reported.lock().unwrap().push(message);
            Box::pin(async { Ok(()) })
        }),
    );
    let feeding = async move {
        let mut left = size;
        while left > 0 {
            let chunk = input.some(CHUNK.min(left as usize)).await?;
            left -= chunk.len() as u64;
            if sender.send(chunk).await.is_err() {
                break;
            }
        }
        Ok::<_, String>(())
    };
    let ((), fed) = futures::join!(writing, feeding);

    let problems = core::mem::take(&mut *problems.lock().unwrap());
    if !problems.is_empty() {
        return Err(format!(
            "'{path}' cannot be written: {}",
            problems.join(", ")
        ));
    }
    fed
}

#[cfg(all(test, feature = "real", unix))]
mod tests {
    use super::*;
    use crate::rooted::RootedFileSystemEngine;
    use futures::executor::block_on;

    /// Directories `source` and `outside` in a base of their own, a rooted file system being
    /// made on the base.
    struct Fixture {
        base: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let base =
                std::env::temp_dir().join(format!("fs-mel-archive-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&base);
            std::fs::create_dir_all(base.join("source/nested/deeper")).unwrap();
            std::fs::create_dir_all(base.join("source/empty")).unwrap();
            std::fs::create_dir_all(base.join("outside")).unwrap();
            Self { base }
        }

        fn engine(&self) -> RootedFileSystemEngine {
            RootedFileSystemEngine::new(self.base.clone())
        }

        fn path(&self, path: &str) -> PathBuf {
            self.base.join(path)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }

    fn tar(
        engine: &RootedFileSystemEngine,
        path: &str,
    ) -> (Result<bool, String>, Vec<u8>, Vec<String>) {
        let mut archive_bytes = Vec::new();
        let mut left_out = Vec::new();
        let result = block_on(archive(
            engine,
            path,
            &mut |bytes| {
                archive_bytes.extend(bytes);
                async { true }
            },
            &mut left_out,
        ));
        (result, archive_bytes, left_out)
    }

    fn untar(engine: &RootedFileSystemEngine, path: &str, data: &[u8]) -> Result<(), String> {
        // Given in small chunks, so that entries span them.
        let mut chunks = data
            .chunks(700)
            .map(|chunk| chunk.to_vec())
            .collect::<VecDeque<_>>();
        block_on(extract(engine, path, &mut || {
            let chunk = chunks.pop_front();
            async move { chunk }
        }))
    }

    fn fill(fixture: &Fixture) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(fixture.path("source/file"), b"content of the file").unwrap();
        std::fs::write(
            fixture.path("source/nested/deeper/big"),
            vec![7u8; 3 * CHUNK + 5],
        )
        .unwrap();
        let long = format!("source/nested/{}", "long-name-".repeat(15));
        std::fs::write(fixture.path(&long), b"long").unwrap();
        std::fs::set_permissions(
            fixture.path("source/file"),
            std::fs::Permissions::from_mode(0o751),
        )
        .unwrap();
        std::os::unix::fs::symlink("../file", fixture.path("source/nested/relative")).unwrap();
        std::os::unix::fs::symlink(
            format!("/{}", "far/".repeat(30)),
            fixture.path("source/absolute"),
        )
        .unwrap();
        let modified = time(1_000_000_000);
        for path in ["source/file", &long, "source/nested/deeper", "source/empty"] {
            std::fs::File::open(fixture.path(path))
                .unwrap()
                .set_modified(modified)
                .unwrap();
        }
    }

    /// Kinds, contents, modes, times and links of the tree of `root`.
    fn tree(root: &Path) -> Vec<(String, String)> {
        use std::os::unix::fs::PermissionsExt;
        let mut entries = Vec::new();
        let mut pending = vec![PathBuf::new()];
        while let Some(relative) = pending.pop() {
            for entry in std::fs::read_dir(root.join(&relative)).unwrap() {
                let entry = entry.unwrap();
                let path = relative.join(entry.file_name());
                let metadata = std::fs::symlink_metadata(root.join(&path)).unwrap();
                let description = if metadata.file_type().is_symlink() {
                    format!(
                        "link to {}",
                        std::fs::read_link(root.join(&path)).unwrap().display()
                    )
                } else if metadata.is_dir() {
                    pending.push(path.clone());
                    format!(
                        "directory {:o} {}",
                        metadata.permissions().mode() & 0o7777,
                        seconds(metadata.modified().ok())
                    )
                } else {
                    format!(
                        "file {:o} {} {:?}",
                        metadata.permissions().mode() & 0o7777,
                        seconds(metadata.modified().ok()),
                        std::fs::read(root.join(&path)).unwrap().len()
                    )
                };
                entries.push((path.to_string_lossy().to_string(), description));
            }
        }
        entries.sort();
        entries
    }

    #[test]
    fn directories_come_back_whole() {
        let fixture = Fixture::new("round-trip");
        fill(&fixture);
        let engine = fixture.engine();

        let (result, archive_bytes, left_out) = tar(&engine, "source");
        assert_eq!(result, Ok(true));
        assert!(left_out.is_empty(), "{left_out:?}");
        assert_eq!(archive_bytes.len() % BLOCK, 0);

        untar(&engine, "copy", &archive_bytes).unwrap();
        assert_eq!(tree(&fixture.path("copy")), tree(&fixture.path("source")));
        assert_eq!(
            std::fs::read(fixture.path("copy/file")).unwrap(),
            b"content of the file"
        );

        // Other tools read the archive too.
        let mut names = ::tar::Archive::new(archive_bytes.as_slice())
            .entries()
            .unwrap()
            .map(|entry| entry.unwrap().path().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert!(names.contains(&"file".to_string()), "{names:?}");
        assert!(
            names
                .iter()
                .any(|name| name.ends_with(&"long-name-".repeat(15))),
            "{names:?}"
        );
        let sorted = {
            names.sort();
            names
        };
        assert!(sorted.len() >= 8);
    }

    #[test]
    fn same_files_give_same_bytes() {
        let fixture = Fixture::new("deterministic");
        fill(&fixture);
        let engine = fixture.engine();
        let (_, first, _) = tar(&engine, "source");
        let (_, second, _) = tar(&engine, "source");
        assert_eq!(first, second);

        // Entries are sorted, whatever the order of the directory.
        let names = ::tar::Archive::new(first.as_slice())
            .entries()
            .unwrap()
            .map(|entry| entry.unwrap().path().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let names = names
            .iter()
            .map(|name| name.trim_end_matches('/'))
            .collect::<Vec<_>>();
        assert_eq!(names[..4], ["absolute", "empty", "file", "nested"]);
    }

    #[test]
    fn other_entries_are_left_out() {
        let fixture = Fixture::new("left-out");
        let _socket =
            std::os::unix::net::UnixListener::bind(fixture.path("source/socket")).unwrap();
        let (result, archive_bytes, left_out) = tar(&fixture.engine(), "source");
        assert_eq!(result, Ok(true));
        assert_eq!(left_out.len(), 1, "{left_out:?}");
        assert!(left_out[0].contains("socket"));
        untar(&fixture.engine(), "copy", &archive_bytes).unwrap();
        assert!(!fixture.path("copy/socket").exists());
    }

    /// Archive of one entry named `name` of `kind`, with `link` as target, and `content`.
    fn hostile(name: &[u8], kind: EntryType, link: Option<&[u8]>, content: &[u8]) -> Vec<u8> {
        let mut bytes = headers(name, kind, content.len() as u64, 0o644, 0, link);
        bytes.extend_from_slice(content);
        bytes.extend(std::iter::repeat(0).take(padding(content.len() as u64)));
        bytes
    }

    #[test]
    fn archives_never_write_out_of_their_target() {
        let fixture = Fixture::new("hostile");
        let engine = fixture.engine();
        let end = vec![0; 2 * BLOCK];

        let cases: Vec<(&str, Vec<u8>)> = vec![
            (
                "parent",
                [
                    hostile(b"../outside/evil", EntryType::Regular, None, b"evil"),
                    end.clone(),
                ]
                .concat(),
            ),
            (
                "nested parent",
                [
                    hostile(b"a/../../outside/evil", EntryType::Regular, None, b"evil"),
                    end.clone(),
                ]
                .concat(),
            ),
            (
                "absolute",
                [
                    hostile(
                        fixture.path("outside/evil").to_string_lossy().as_bytes(),
                        EntryType::Regular,
                        None,
                        b"evil",
                    ),
                    end.clone(),
                ]
                .concat(),
            ),
            (
                "through a link",
                [
                    hostile(
                        b"link",
                        EntryType::Symlink,
                        Some(fixture.path("outside").to_string_lossy().as_bytes()),
                        b"",
                    ),
                    hostile(b"link/evil", EntryType::Regular, None, b"evil"),
                    end.clone(),
                ]
                .concat(),
            ),
            (
                "through a relative link",
                [
                    hostile(b"link", EntryType::Symlink, Some(b"../outside"), b""),
                    hostile(b"link/evil", EntryType::Regular, None, b"evil"),
                    end.clone(),
                ]
                .concat(),
            ),
            (
                "hard link",
                [
                    hostile(b"hard", EntryType::Link, Some(b"../outside/evil"), b""),
                    end.clone(),
                ]
                .concat(),
            ),
            (
                "device",
                [hostile(b"device", EntryType::Char, None, b""), end.clone()].concat(),
            ),
        ];
        for (index, (case, data)) in cases.into_iter().enumerate() {
            let target = format!("target-{index}");
            assert!(untar(&engine, &target, &data).is_err(), "{case}");
            assert_eq!(
                std::fs::read_dir(fixture.path("outside")).unwrap().count(),
                0,
                "{case}"
            );
        }
    }

    #[test]
    fn existing_targets_and_cut_archives_fail() {
        let fixture = Fixture::new("failures");
        fill(&fixture);
        let engine = fixture.engine();
        let (_, archive_bytes, _) = tar(&engine, "source");

        assert_eq!(
            untar(&engine, "source", &archive_bytes),
            Err("'source' already exists".to_string())
        );
        assert!(untar(&engine, "cut", &archive_bytes[..archive_bytes.len() / 2]).is_err());
        assert!(untar(&engine, "garbage", &[1u8; 2 * BLOCK]).is_err());
        assert!(tar(&engine, "source/file").0.is_err());
    }

    #[test]
    fn pax_headers_are_read() {
        // Each record counts its own length.
        let record = |key: &str, value: &str| {
            let rest = format!(" {key}={value}\n");
            let mut length = rest.len() + 1;
            while format!("{length}").len() + rest.len() != length {
                length += 1;
            }
            format!("{length}{rest}")
        };
        let records = [
            record("path", "pax/named"),
            record("mtime", "1000000000.123456789"),
        ]
        .concat();
        let records = records.as_bytes();
        let data = [
            hostile(b"PaxHeaders/x", EntryType::XHeader, None, records),
            hostile(b"ignored", EntryType::Regular, None, b"pax"),
            vec![0; 2 * BLOCK],
        ]
        .concat();
        let fixture = Fixture::new("pax");
        untar(&fixture.engine(), "copy", &data).unwrap();
        assert_eq!(
            std::fs::read(fixture.path("copy/pax/named")).unwrap(),
            b"pax"
        );
        assert_eq!(
            seconds(
                std::fs::metadata(fixture.path("copy/pax/named"))
                    .unwrap()
                    .modified()
                    .ok()
            ),
            1_000_000_000
        );
    }
}
