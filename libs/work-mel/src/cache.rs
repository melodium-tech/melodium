//! Caches of Cadence.CI: directories of the volumes of a worker saved under a key, restored
//! by later runs of the organization, such as dependencies or build outputs.
//!
//! A cache is an archive, immutable once saved: the first run saving a key keeps it, and
//! saving an existing key uploads nothing. Keys therefore change with what they hold, such
//! as `cargo-` followed by a hash of `Cargo.lock`. Restoring tries each key in turn, as is,
//! then as the prefix of the newest cache whose key starts with it, so that a cache close
//! to the one wanted is restored when the exact one does not exist yet.
//!
//! Caches only exist for programs run by Cadence.CI, on its workers: the API is reached
//! as configured there, with the token of the run, and volumes are those of the worker.
//! A missed cache is never an error.

#[cfg(feature = "real")]
use crate::run::{answer, refusal, RunApi};
#[cfg(feature = "real")]
use crate::storage::{transfer, volume_path, volume_root, Completion, PresignedRequest, Upload};
use melodium_core::common::executive::{Level, SecretsAccess, TrackId, World};
use melodium_core::*;
use melodium_macro::{mel_data, mel_function, mel_treatment};
use std::sync::{Arc, Weak};
#[cfg(feature = "real")]
use uuid::Uuid;

/// Label of the log entries of caches.
const LOG_LABEL: &str = "cache";

/// Cache of Cadence.CI for a directory of a volume, saved under `key`, and restored by `key`
/// or by `restore_keys`, tried in turn, as `restore` tells.
///
/// Built with `|cache`, it is restored by `restoreCache` and saved by `saveCache`, as steps
/// of `cicd` do with their `cache`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[mel_data(traits(Serialize Deserialize PartialEquality Equality))]
pub struct Cache {
    pub volume: string,
    pub path: string,
    pub key: string,
    pub restore_keys: Vec<string>,
}

/// Build a `Cache` of `path` in `volume`, saved under `key`, restored by `key` then by
/// `restore_keys`, such as `|cache("data", "cargo", "cargo-<hash of Cargo.lock>", ["cargo-"])`.
#[mel_function]
pub fn cache(volume: string, path: string, key: string, restore_keys: Vec<string>) -> Cache {
    Cache {
        volume,
        path,
        key,
        restore_keys,
    }
}

/// Restores a cache of Cadence.CI into a directory of a volume.
///
/// Each key of `keys` is tried in turn: first a cache saved under that exact key, then
/// the newest cache whose key starts with it. The matching cache is extracted into `path`
/// of `volume`, `path` being relative to the volume and created if needed. Files of
/// the cache replace those with the same name, other files are kept.
///
/// `restored` is emitted once a cache is restored, and `matched` gives its key, compared
/// with the key it will be saved under to know whether it needs saving again. Otherwise
/// `missed` is emitted: no cache matched, or it could not be restored, which is logged as
/// a warning, never as an error. `finished` is emitted in any case.
///
/// The archive is checked against the sha256 recorded when it was saved before anything is
/// extracted: an archive altered in storage is never extracted, and ends on `missed`.
/// While it is downloaded, the archive is kept in the volume, in a file without name: the
/// volume needs room for it.
///
/// ```mermaid
/// graph LR
///     T("restore()")
///     K["〈🟦〉"] -->|keys| T
///     T -->|restored| R["〈🟩〉"]
///     T -->|matched| M["〈🟨〉"]
///     T -->|missed| X["〈🟥〉"]
///     T -->|finished| F["〈🟦〉"]
///     style K fill:#ffffff,stroke:#ffffff
///     style R fill:#ffffff,stroke:#ffffff
///     style M fill:#ffffff,stroke:#ffffff
///     style X fill:#ffffff,stroke:#ffffff
///     style F fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input keys Block<Vec<string>>
    output restored Block<void>
    output matched Block<string>
    output missed Block<void>
    output finished Block<void>
)]
pub async fn restore(volume: string, path: string) {
    if let Ok(keys) = keys.recv_one_as::<Vec<String>>().await {
        match restore_logged(&world, track_id, &volume, &path, &keys).await {
            Some(key) => {
                let _ = restored.send_one(().into()).await;
                let _ = matched.send_one(key.into()).await;
            }
            None => {
                let _ = missed.send_one(().into()).await;
            }
        }
        let _ = finished.send_one(().into()).await;
    }
}

/// Restores a `Cache` of Cadence.CI, trying its `key`, then its `restore_keys`.
///
/// Same as `restore`, with the cache given as a block, as steps of `cicd` do.
#[mel_treatment(
    input cache Block<Cache>
    output restored Block<void>
    output matched Block<string>
    output missed Block<void>
    output finished Block<void>
)]
pub async fn restoreCache() {
    if let Ok(cache) = cache.recv_one_as::<Arc<Cache>>().await {
        let keys = std::iter::once(cache.key.clone())
            .chain(cache.restore_keys.iter().cloned())
            .filter(|key| !key.is_empty())
            .collect::<Vec<_>>();
        match restore_logged(&world, track_id, &cache.volume, &cache.path, &keys).await {
            Some(key) => {
                let _ = restored.send_one(().into()).await;
                let _ = matched.send_one(key.into()).await;
            }
            None => {
                let _ = missed.send_one(().into()).await;
            }
        }
        let _ = finished.send_one(().into()).await;
    }
}

/// Saves a directory of a volume as a cache of Cadence.CI, for later runs to restore it.
///
/// The content of `path` of `volume` is archived and saved under `key`, for the runs of the
/// organization. Links are kept as links, never followed. A cache is never replaced: when
/// `key` already exists, nothing is uploaded, which is also a success. Saving a cache that
/// was restored under the same key archives it for nothing, `restore` telling the key it
/// `matched`, and `saveCache` skipping it.
///
/// The archive is refused when it holds a value revealed from a secret by this engine, in
/// a file or in a name, as is, in base64 or URL-encoded. While it is uploaded, the archive
/// is kept in the volume, in a file without name: the volume needs room for it.
///
/// `completed` is emitted once the cache is saved, or already existed. Otherwise `failed` is
/// emitted, and `error` gives the reason, such as Cadence.CI skipping the cache when the
/// storage of the organization is full. `finished` is emitted in any case.
#[mel_treatment(
    input key Block<string>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    output finished Block<void>
    secrets_access
)]
pub async fn save(volume: string, path: string) {
    if let Ok(key) = key.recv_one_as::<String>().await {
        match save_logged(&secrets_access, &world, track_id, &volume, &path, &key).await {
            Ok(()) => {
                let _ = completed.send_one(().into()).await;
            }
            Err(problem) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(problem.into()).await;
            }
        }
        let _ = finished.send_one(().into()).await;
    }
}

/// Saves a `Cache` of Cadence.CI under its `key`, unless it was restored under that key.
///
/// Same as `save`, with the cache given as a block, as steps of `cicd` do. When `matched`
/// gives the `key` of the cache, as `restoreCache` does when it restored that exact key,
/// nothing is archived and `completed` is emitted.
#[mel_treatment(
    input cache Block<Cache>
    input matched Block<string>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    output finished Block<void>
    secrets_access
)]
pub async fn saveCache() {
    if let Ok(cache) = cache.recv_one_as::<Arc<Cache>>().await {
        let outcome = if matched.recv_one_as::<String>().await.ok().as_ref() == Some(&cache.key) {
            log(
                &world,
                track_id,
                Level::Info,
                format!(
                    "Cache '{}' was restored under its key, not saved again",
                    cache.key
                ),
            )
            .await;
            Ok(())
        } else {
            save_logged(
                &secrets_access,
                &world,
                track_id,
                &cache.volume,
                &cache.path,
                &cache.key,
            )
            .await
        };
        match outcome {
            Ok(()) => {
                let _ = completed.send_one(().into()).await;
            }
            Err(problem) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(problem.into()).await;
            }
        }
        let _ = finished.send_one(().into()).await;
    }
}

/// Logs `message` about caches.
async fn log(world: &Weak<dyn World>, track_id: TrackId, level: Level, message: String) {
    if let Some(world) = world.upgrade() {
        world
            .log(level, LOG_LABEL.to_string(), message, Some(track_id))
            .await;
    }
}

/// Restores into `path` of `volume` the first cache matching `keys`, giving its key.
///
/// A cache that cannot be restored is logged as a warning, and given as missed.
async fn restore_logged(
    world: &Weak<dyn World>,
    track_id: TrackId,
    volume: &str,
    path: &str,
    keys: &[String],
) -> Option<String> {
    match restoring(volume, path, keys).await {
        Ok(Some(key)) => {
            log(
                world,
                track_id,
                Level::Info,
                format!("Cache '{key}' restored into '{path}' of volume '{volume}'"),
            )
            .await;
            Some(key)
        }
        Ok(None) => None,
        Err(problem) => {
            log(
                world,
                track_id,
                Level::Warning,
                format!("No cache restored into '{path}' of volume '{volume}': {problem}"),
            )
            .await;
            None
        }
    }
}

/// Saves `path` of `volume` as the cache `key`, logging what happened, or giving why it
/// could not be saved.
async fn save_logged(
    access: &SecretsAccess,
    world: &Weak<dyn World>,
    track_id: TrackId,
    volume: &str,
    path: &str,
    key: &str,
) -> Result<(), String> {
    match saving(access, volume, path, key).await {
        Ok(uploaded) => {
            log(
                world,
                track_id,
                Level::Info,
                if uploaded {
                    format!("Cache '{key}' saved from '{path}' of volume '{volume}'")
                } else {
                    format!("Cache '{key}' already exists, nothing uploaded")
                },
            )
            .await;
            Ok(())
        }
        Err(problem) => Err(format!(
            "Cache '{key}' not saved from '{path}' of volume '{volume}': {problem}"
        )),
    }
}

/// Request restoring a cache, with the keys to try in order.
#[cfg(feature = "real")]
#[derive(Serialize)]
struct RestoreRequest<'a> {
    keys: &'a [String],
}

/// Cache matching a restore request, if any (`CacheRestoreResponse` of the API).
#[cfg(feature = "real")]
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Restore {
    Hit {
        key: String,
        size: u64,
        sha256: String,
        download: PresignedRequest,
    },
    Miss,
}

/// Request saving a cache, reserving its size.
#[cfg(feature = "real")]
#[derive(Serialize)]
struct SaveRequest<'a> {
    key: &'a str,
    size: u64,
}

/// What to do to save a cache (`CacheSaveResponse` of the API).
#[cfg(feature = "real")]
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Save {
    /// A cache already exists under that key.
    Exists,
    /// The archive is to be uploaded as `upload` tells, then completed.
    Upload { object_id: Uuid, upload: Upload },
    /// Not saved, such as when the storage of the organization is full.
    Skipped { reason: String },
}

/// Restores the first cache matching `keys` into `path` of `volume`, giving its key.
#[cfg(feature = "real")]
async fn restoring(volume: &str, path: &str, keys: &[String]) -> Result<Option<String>, String> {
    if keys.is_empty() {
        return Ok(None);
    }
    let api = RunApi::from_environment().map_err(|problem| {
        format!("caches are only given to programs run by Cadence.CI, and {problem}")
    })?;
    let target = volume_path(volume, path)?;
    let root = volume_root(volume)?;

    let (key, size, sha256, download) = match answer(
        api.post_json(api.run_url("cache/restore"), &RestoreRequest { keys })
            .await?,
    )? {
        Restore::Hit {
            key,
            size,
            sha256,
            download,
        } => (key, size, sha256, download),
        Restore::Miss => return Ok(None),
    };

    // Engines have little storage of their own: the archive goes to the volume, in a file
    // without name, never seen by executors.
    let file = tempfile::tempfile_in(root)
        .map_err(|err| format!("cache '{key}' cannot be downloaded: {err}"))?;
    let file = transfer::download(&download, size, &sha256, file)
        .await
        .map_err(|problem| format!("cache '{key}' not restored, {problem}"))?;
    smol::unblock(move || crate::storage::archive::unpack(file, &target))
        .await
        .map_err(|problem| {
            format!("cache '{key}' could only be partly restored, '{path}' may hold some of it: {problem}")
        })?;
    Ok(Some(key))
}

#[cfg(feature = "mock")]
async fn restoring(_volume: &str, _path: &str, _keys: &[String]) -> Result<Option<String>, String> {
    Err("caches cannot be restored in this build".to_string())
}

/// Saves `path` of `volume` as the cache `key`, telling whether it was uploaded.
#[cfg(feature = "real")]
async fn saving(
    access: &SecretsAccess,
    volume: &str,
    path: &str,
    key: &str,
) -> Result<bool, String> {
    if key.is_empty() {
        return Err("caches need a key".to_string());
    }
    let api = RunApi::from_environment().map_err(|problem| {
        format!("caches are only saved by programs run by Cadence.CI, and {problem}")
    })?;
    let source = volume_path(volume, path)?;
    if !source.is_dir() {
        return Err("it is not a directory".to_string());
    }

    // The token of the run is not a secret of the program, but must not leave in a cache.
    access.add_masked_value(
        "Cadence.CI run token",
        &Value::String(api.token.to_string()),
    );
    let scan = access.revealed_scan();
    // Engines have little storage of their own: the archive goes to the volume, in a file
    // without name, never archived itself.
    let file = tempfile::tempfile_in(volume_root(volume)?).map_err(|err| err.to_string())?;
    let (file, size) =
        smol::unblock(move || crate::storage::archive::pack(&source, file, scan)).await?;

    let (object_id, upload) = match answer(
        api.post_json(api.run_url("cache/save"), &SaveRequest { key, size })
            .await?,
    )? {
        Save::Exists => return Ok(false),
        Save::Upload { object_id, upload } => (object_id, upload),
        Save::Skipped { reason } => return Err(format!("Cadence.CI skipped it: {reason}")),
    };

    let (sha256, parts) =
        transfer::upload(Arc::new(std::sync::Mutex::new(file)), size, &upload).await?;
    let (status, body) = api
        .post_json(
            api.run_url(&format!("cache/save/{object_id}/complete")),
            &Completion { sha256, parts },
        )
        .await?;
    if status.is_success() {
        Ok(true)
    } else if status == reqwest::StatusCode::CONFLICT {
        // Another run saved that key meanwhile, and keeps it.
        Ok(false)
    } else {
        Err(refusal(status, &body))
    }
}

#[cfg(feature = "mock")]
async fn saving(
    _access: &SecretsAccess,
    _volume: &str,
    _path: &str,
    _key: &str,
) -> Result<bool, String> {
    Err("caches cannot be saved in this build".to_string())
}

/// Answers as `melodium-service-api` serializes them, and requests as it parses them.
#[cfg(all(test, feature = "real"))]
mod contract_tests {
    use super::*;

    #[test]
    fn answers_of_the_api_are_understood() {
        let hit = r#"{"hit":{"key":"cargo-1","size":10,"sha256":"ab","download":{"method":"GET","url":"https://s/c","headers":{"x-amz-meta":"v"}}}}"#;
        match serde_json::from_str::<Restore>(hit).unwrap() {
            Restore::Hit {
                key,
                size,
                sha256,
                download,
            } => {
                assert_eq!((key.as_str(), size, sha256.as_str()), ("cargo-1", 10, "ab"));
                assert_eq!(download.headers["x-amz-meta"], "v");
            }
            Restore::Miss => panic!("hit expected"),
        }
        assert!(matches!(
            serde_json::from_str::<Restore>(r#""miss""#).unwrap(),
            Restore::Miss
        ));

        let upload = r#"{"upload":{"object_id":"00000000-0000-0000-0000-000000000001","upload":{"multipart":{"part_size":5,"parts":[{"method":"PUT","url":"https://s/1","headers":{"x-amz-meta":"v"}},{"method":"PUT","url":"https://s/2","headers":{"x-amz-meta":"v"}}]}}}}"#;
        match serde_json::from_str::<Save>(upload).unwrap() {
            Save::Upload {
                upload: Upload::Multipart { part_size, parts },
                ..
            } => {
                assert_eq!((part_size, parts.len()), (5, 2))
            }
            _ => panic!("multipart upload expected"),
        }
        assert!(matches!(
            serde_json::from_str::<Save>(r#""exists""#).unwrap(),
            Save::Exists
        ));
        match serde_json::from_str::<Save>(r#"{"skipped":{"reason":"storage full"}}"#).unwrap() {
            Save::Skipped { reason } => assert_eq!(reason, "storage full"),
            _ => panic!("skipped expected"),
        }
    }

    #[test]
    fn requests_are_those_of_the_api() {
        let keys = vec!["cargo-1".to_string()];
        assert_eq!(
            serde_json::to_string(&RestoreRequest { keys: &keys }).unwrap(),
            r#"{"keys":["cargo-1"]}"#
        );
        assert_eq!(
            serde_json::to_string(&SaveRequest {
                key: "cargo-1",
                size: 10
            })
            .unwrap(),
            r#"{"key":"cargo-1","size":10}"#
        );
        let completion = Completion {
            sha256: "ab".to_string(),
            parts: vec![crate::storage::UploadedPart {
                part_number: 1,
                etag: "\"e\"".to_string(),
            }],
        };
        assert_eq!(
            serde_json::to_string(&completion).unwrap(),
            r#"{"sha256":"ab","parts":[{"part_number":1,"etag":"\"e\""}]}"#
        );
    }
}
