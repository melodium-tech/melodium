use async_trait::async_trait;
use core::{future::Future, pin::Pin};
use melodium_core::*;
use melodium_macro::mel_data;
use std::{fmt::Debug, sync::Arc, time::SystemTime};

pub type OnceTriggerCall<'a> =
    Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send + Sync + 'a>;
pub type OnceMessageCall<'a> =
    Box<dyn FnOnce(String) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send + Sync + 'a>;
pub type OutU128Call<'a> = Box<
    dyn Fn(u128) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>> + Send + Sync + 'a,
>;
pub type OutMessageCall<'a> = Box<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>> + Send + Sync + 'a,
>;
pub type InDataCall<'a> = Box<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<Vec<u8>, ()>> + Send + 'a>> + Send + Sync + 'a,
>;
pub type OutDataCall<'a> = Box<
    dyn Fn(VecDeque<u8>) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>>
        + Send
        + Sync
        + 'a,
>;

/// Kind of an entry of a file system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
    /// Symbolic link, never followed.
    Link,
    /// Device, socket, pipe, or anything else.
    Other,
}

/// What a file system tells about an entry, links not being followed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub kind: EntryKind,
    /// Size in bytes, of the content for files.
    pub size: u64,
    /// Permissions mode, such as `0o755`, where the file system has one.
    pub mode: Option<u32>,
    pub modified: Option<SystemTime>,
}

#[async_trait]
pub trait FileSystemEngine: Debug + Send + Sync {
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
    );
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
    );
    async fn create_dir(
        &self,
        path: &str,
        recursive: bool,
        success: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
    );
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
    );
    /// Tells what `path` is, without following it if it is a link.
    async fn metadata(&self, path: &str) -> Result<Metadata, String>;
    /// Gives the names of the entries of the directory `path`, in no particular order.
    async fn read_dir(&self, path: &str) -> Result<Vec<String>, String>;
    /// Gives the target of the link `path`, as written in it, never resolved.
    async fn read_link(&self, path: &str) -> Result<String, String>;
    /// Creates a link at `path` to `target`, written as is, never resolved.
    async fn create_link(&self, target: &str, path: &str) -> Result<(), String>;
    /// Sets the permissions `mode` and modification time of `path`, when given. A link keeps
    /// its own, and what it points to is left unchanged.
    async fn set_metadata(
        &self,
        path: &str,
        mode: Option<u32>,
        modified: Option<SystemTime>,
    ) -> Result<(), String>;
    /// Removes the file or link `path` (never what a link points to), the empty directory
    /// `path`, or the whole tree of `path` when `recursive`.
    async fn remove(&self, path: &str, recursive: bool) -> Result<(), String>;
    /// Moves the file or directory `from` to `to`, within the file system.
    async fn rename(&self, from: &str, to: &str) -> Result<(), String>;
}

#[derive(Debug, Serialize)]
/// Abstract handle to a filesystem implementation.
///
/// `FileSystem` carries a filesystem engine (local, remote, container, etc.)
/// and is consumed by filesystem treatments such as `readFile`, `writeFile`, `scanDir`, and `createDir`.
/// Obtain a `FileSystem` value via companion functions such as `|local_filesystem`.
#[mel_data]
pub struct FileSystem {
    #[serde(skip)]
    pub filesystem: Arc<dyn FileSystemEngine>,
}
