//! Run artifacts of Cadence.CI: what a run produced, kept and shown, such as deliverables,
//! reports, or renders shown in the run page.
//!
//! An artifact is a file tree, or a single file, named within its run. It has a stable
//! address, `<API>/artifacts/<run>/<name>`, giving the file itself for an artifact of one
//! file and the list of its files otherwise, and `<API>/artifacts/<run>/<name>/<path>` for
//! each file. It is kept for the retention of the plan of the organization, three months at
//! least, or as long as the program asks, and is seen by the members of the organization,
//! or by anyone when its access is `public`, whatever the access of the run.
//!
//! Artifacts keep and show what a run produced: the steps of a program pass data to each
//! other as streams, not through artifacts.
//!
//! Artifacts only exist for programs run by Cadence.CI, on its workers: the API is reached
//! as configured there, with the token of the run, and volumes are those of the worker.

#[cfg(feature = "real")]
use crate::run::{answer, refusal, RunApi};
#[cfg(feature = "real")]
use crate::storage::{transfer, tree, volume_path, PresignedRequest, Upload, UploadedPart};
use melodium_core::common::executive::{Level, SecretsAccess, TrackId, World};
use melodium_core::*;
use melodium_macro::{mel_data, mel_function, mel_treatment};
use std::sync::{Arc, Weak};

/// Label of the log entries of artifacts.
const LOG_LABEL: &str = "artifact";
/// Files transferred at the same time.
#[cfg(feature = "real")]
const PARALLEL_TRANSFERS: usize = 4;
/// Longest artifact name.
const MAX_NAME_LENGTH: usize = 255;

/// Run artifact to push: the file or directory `path` of `volume`, named `name` in its run.
///
/// Built with `|artifact`, and changed with `|with_access`, `|with_retention`,
/// `|without_expiry`, `|with_entrypoint` and `|locked`, it is pushed by `pushArtifacts`, as
/// steps of `cicd` do with their `artifacts`. `push` tells what each field means.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[mel_data(traits(Serialize Deserialize PartialEquality Equality))]
pub struct Artifact {
    pub volume: string,
    pub path: string,
    pub name: string,
    pub access: string,
    pub retention: Option<u32>,
    pub expires: bool,
    pub entrypoint: Option<string>,
    pub lock: bool,
}

/// Build an `Artifact` of `path` in `volume`, a file or a directory, named `name` in its run.
///
/// It is seen by the members of the organization, kept for the retention of its plan, three
/// months at least, and can be deleted.
#[mel_function]
pub fn artifact(volume: string, path: string, name: string) -> Artifact {
    Artifact {
        volume,
        path,
        name,
        access: "organization".to_string(),
        retention: None,
        expires: true,
        entrypoint: None,
        lock: false,
    }
}

/// Gives `artifact` the `access` given: `"organization"` for the members of the organization
/// only, or `"public"` for anyone.
#[mel_function]
pub fn with_access(mut artifact: Artifact, access: string) -> Artifact {
    artifact.access = access;
    artifact
}

/// Keeps `artifact` for `days` days from its push, instead of the retention of the plan.
#[mel_function]
pub fn with_retention(mut artifact: Artifact, days: u32) -> Artifact {
    artifact.retention = Some(days);
    artifact.expires = true;
    artifact
}

/// Keeps `artifact` until it is deleted.
#[mel_function]
pub fn without_expiry(mut artifact: Artifact) -> Artifact {
    artifact.retention = None;
    artifact.expires = false;
    artifact
}

/// Makes `artifact` a render shown in the run page, opening `entrypoint`, the path of one of
/// its files, such as `index.html`.
#[mel_function]
pub fn with_entrypoint(mut artifact: Artifact, entrypoint: string) -> Artifact {
    artifact.entrypoint = Some(entrypoint);
    artifact
}

/// Locks `artifact`: it can then never be deleted, nor kept for a shorter time.
#[mel_function]
pub fn locked(mut artifact: Artifact) -> Artifact {
    artifact.lock = true;
    artifact
}

/// Pushes a file or a directory of a volume as a run artifact of Cadence.CI.
///
/// `path` of `volume` is pushed as the artifact `name` of the run, `path` being relative to
/// the volume: a file is an artifact of one file, named as it is, and a directory gives its
/// files and those of its subdirectories, at their path within it. Links are not followed:
/// they are left out, with a warning. Names are made of letters, digits, `_`, `.` and `-`,
/// and are unique within the run.
///
/// - `access`: `"organization"` for the members of the organization only, or `"public"` for
///   anyone, whatever the access of the run;
/// - `retention`: days the artifact is kept from its push, none for the retention of the
///   plan of the organization, three months at least;
/// - `expires`: false to keep the artifact until it is deleted, `retention` being none;
/// - `entrypoint`: path of one of the files, such as `index.html`, making the artifact a
///   render shown in the run page; renders run on a domain of their own, reach the other
///   artifacts of the run by relative paths, such as `../metrics/data.json`, and have a
///   size limit;
/// - `lock`: true to keep the artifact from ever being deleted or kept for a shorter time.
///
/// Files are read whole before anything is sent: the artifact is refused when a file, or its
/// path, holds a value revealed from a secret by this engine, as is, in base64 or
/// URL-encoded. They are sent to storage in parallel, each with its sha256, which `fetch`
/// checks.
///
/// `completed` is emitted once the artifact is pushed, and `address` gives its stable
/// address. Otherwise `failed` is emitted, and `error` gives the reason. `finished` is
/// emitted in any case.
#[mel_treatment(
    default access "organization"
    default retention none
    default expires true
    default entrypoint none
    default lock false
    input trigger Block<void>
    output completed Block<void>
    output address Block<string>
    output failed Block<void>
    output error Block<string>
    output finished Block<void>
    secrets_access
)]
pub async fn push(
    volume: string,
    path: string,
    name: string,
    access: string,
    retention: Option<u32>,
    expires: bool,
    entrypoint: Option<string>,
    lock: bool,
) {
    if let Ok(_) = trigger.recv_one().await {
        let artifact = Artifact {
            volume,
            path,
            name,
            access,
            retention,
            expires,
            entrypoint,
            lock,
        };
        match push_logged(&secrets_access, &world, track_id, &artifact).await {
            Ok(stable_address) => {
                let _ = completed.send_one(().into()).await;
                let _ = address.send_one(stable_address.into()).await;
            }
            Err(problem) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(problem.into()).await;
            }
        }
        let _ = finished.send_one(().into()).await;
    }
}

/// Pushes `artifacts` as run artifacts of Cadence.CI, one after the other.
///
/// Same as `push` for each artifact, as steps of `cicd` do. `completed` is emitted once every
/// artifact is pushed. Otherwise `failed` is emitted, and `errors` gives the reason for each
/// artifact not pushed, the others being pushed. `finished` is emitted in any case.
#[mel_treatment(
    input artifacts Block<Vec<Artifact>>
    output completed Block<void>
    output failed Block<void>
    output errors Stream<string>
    output finished Block<void>
    secrets_access
)]
pub async fn pushArtifacts() {
    if let Ok(artifacts) = artifacts.recv_one_as::<Vec<Arc<Artifact>>>().await {
        let mut problems = Vec::new();
        for artifact in &artifacts {
            if let Err(problem) = push_logged(&secrets_access, &world, track_id, artifact).await {
                problems.push(problem);
            }
        }
        if problems.is_empty() {
            let _ = completed.send_one(().into()).await;
        } else {
            let _ = failed.send_one(().into()).await;
            let _ = errors
                .send_many(TransmissionValue::String(problems.into()))
                .await;
        }
        let _ = finished.send_one(().into()).await;
    }
}

/// Fetches a run artifact of Cadence.CI into a directory of a volume.
///
/// The artifact `name` of the run `run`, a run of the organization given by its identifier,
/// is written into `path` of `volume`, `path` being relative to the volume and created if
/// needed: each file goes at its path within the artifact, replacing the file there, other
/// files being kept. Files are written without execution permission.
///
/// Every file is checked against the sha256 recorded when it was pushed before any is put in
/// place: on a mismatch, or any other failure, nothing written remains.
///
/// `completed` is emitted once the artifact is fetched. Otherwise `failed` is emitted, and
/// `error` gives the reason, such as the artifact having expired. `finished` is emitted in
/// any case.
#[mel_treatment(
    input trigger Block<void>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    output finished Block<void>
)]
pub async fn fetch(volume: string, path: string, run: string, name: string) {
    if let Ok(_) = trigger.recv_one().await {
        match fetching(&volume, &path, &run, &name).await {
            Ok(count) => {
                log(
                    &world,
                    track_id,
                    Level::Info,
                    format!(
                        "Artifact '{name}' of run {run} fetched into '{path}' of volume '{volume}', {count} file{}",
                        if count == 1 { "" } else { "s" }
                    ),
                )
                .await;
                let _ = completed.send_one(().into()).await;
            }
            Err(problem) => {
                let _ = failed.send_one(().into()).await;
                let _ = error
                    .send_one(
                        format!("Artifact '{name}' of run {run} not fetched: {problem}").into(),
                    )
                    .await;
            }
        }
        let _ = finished.send_one(().into()).await;
    }
}

/// Logs `message` about artifacts.
async fn log(world: &Weak<dyn World>, track_id: TrackId, level: Level, message: String) {
    if let Some(world) = world.upgrade() {
        world
            .log(level, LOG_LABEL.to_string(), message, Some(track_id))
            .await;
    }
}

/// Pushes `artifact`, logging what happened, and gives its stable address, or why it
/// could not be pushed.
async fn push_logged(
    access: &SecretsAccess,
    world: &Weak<dyn World>,
    track_id: TrackId,
    artifact: &Artifact,
) -> Result<String, String> {
    match pushing(access, artifact).await {
        Ok((address, links)) => {
            for link in links {
                log(
                    world,
                    track_id,
                    Level::Warning,
                    format!(
                        "'{link}' is a link, left out of artifact '{}': artifacts hold files only",
                        artifact.name
                    ),
                )
                .await;
            }
            log(
                world,
                track_id,
                Level::Info,
                format!("Artifact '{}' pushed, at {address}", artifact.name),
            )
            .await;
            Ok(address)
        }
        Err(problem) => Err(format!(
            "Artifact '{}' not pushed from '{}' of volume '{}': {problem}",
            artifact.name, artifact.path, artifact.volume
        )),
    }
}

/// Checks that `name` is an artifact name: letters, digits, `_`, `.` and `-`.
fn check_name(name: &str) -> Result<(), String> {
    if !name.is_empty()
        && name.len() <= MAX_NAME_LENGTH
        && name != "."
        && name != ".."
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
    {
        Ok(())
    } else {
        Err(format!(
            "'{name}' is not an artifact name, made of letters, digits, '_', '.' and '-'"
        ))
    }
}

/// How long an artifact is kept (`ArtifactRetention` of the API), none being the retention of
/// the plan, three months at least.
#[cfg(feature = "real")]
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Retention {
    /// Until deleted.
    Never,
    Days(u32),
}

/// File of an artifact to push.
#[cfg(feature = "real")]
#[derive(Serialize)]
struct PushedFile<'a> {
    path: &'a str,
    size: u64,
    media_type: String,
}

/// Request pushing an artifact (`ArtifactPushRequest` of the API).
#[cfg(feature = "real")]
#[derive(Serialize)]
struct PushRequest<'a> {
    name: &'a str,
    files: Vec<PushedFile<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entrypoint: Option<&'a str>,
    access: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    retention: Option<Retention>,
    lock: bool,
}

/// Artifact reserved, with how to upload each of its files (`ArtifactPushResponse` of the
/// API).
#[cfg(feature = "real")]
#[derive(Deserialize)]
struct Pushed {
    artifact_id: uuid::Uuid,
    uploads: Vec<FileUpload>,
}

#[cfg(feature = "real")]
#[derive(Deserialize)]
struct FileUpload {
    path: String,
    upload: Upload,
}

/// Completion of the upload of an artifact.
#[cfg(feature = "real")]
#[derive(Serialize)]
struct ArtifactCompletion {
    files: Vec<CompletedFile>,
}

#[cfg(feature = "real")]
#[derive(Serialize)]
struct CompletedFile {
    path: String,
    sha256: String,
    parts: Vec<UploadedPart>,
}

/// Files of an artifact, as its stable address gives them with `?manifest` (`ArtifactDownload`
/// of the API).
#[cfg(feature = "real")]
#[derive(Deserialize)]
struct Manifest {
    files: Vec<ManifestFile>,
}

#[cfg(feature = "real")]
#[derive(Deserialize)]
struct ManifestFile {
    path: String,
    size: u64,
    /// Recorded when the file was pushed, so always given for complete artifacts.
    sha256: Option<String>,
    download: PresignedRequest,
}

/// Pushes `artifact`, giving its stable address, and the links left out.
#[cfg(feature = "real")]
async fn pushing(
    secrets: &SecretsAccess,
    artifact: &Artifact,
) -> Result<(String, Vec<String>), String> {
    use futures::{StreamExt, TryStreamExt};

    check_name(&artifact.name)?;
    if !matches!(artifact.access.as_str(), "organization" | "public") {
        return Err(format!(
            "'{}' is not an access, being 'organization' or 'public'",
            artifact.access
        ));
    }
    let retention = match (artifact.retention, artifact.expires) {
        (None, true) => None,
        (Some(days), true) => Some(Retention::Days(days)),
        (None, false) => Some(Retention::Never),
        (Some(_), false) => {
            return Err("an artifact kept until deleted has no retention".to_string())
        }
    };
    if let Some(entrypoint) = &artifact.entrypoint {
        tree::check_path(entrypoint)?;
    }
    let api = RunApi::from_environment().map_err(|problem| {
        format!("artifacts are only pushed by programs run by Cadence.CI, and {problem}")
    })?;
    let source = volume_path(&artifact.volume, &artifact.path)?;

    let (files, links) = smol::unblock(move || tree::list(&source)).await?;
    if files.is_empty() {
        return Err("there is no file to push".to_string());
    }
    for file in &files {
        tree::check_path(&file.path)?;
    }
    if let Some(entrypoint) = &artifact.entrypoint {
        if !files.iter().any(|file| &file.path == entrypoint) {
            return Err(format!(
                "its entrypoint '{entrypoint}' is not one of its files"
            ));
        }
    }

    // The token of the run is not a secret of the program, but must not leave in a file.
    secrets.add_masked_value(
        "Cadence.CI run token",
        &Value::String(api.token.to_string()),
    );
    for file in &files {
        if let Some(secret) = secrets.revealed_scan().scan(file.path.as_bytes()) {
            return Err(format!(
                "a path holds the value of secret '{secret}' revealed in this run"
            ));
        }
    }
    let scans = files
        .iter()
        .map(|_| secrets.revealed_scan())
        .collect::<Vec<_>>();
    let (files, hashes) = smol::unblock(move || {
        let mut hashes = Vec::with_capacity(files.len());
        for (file, mut scan) in files.iter().zip(scans) {
            match tree::hash_and_scan(file, &mut scan)? {
                Ok(sha256) => hashes.push(sha256),
                Err(secret) => {
                    return Err(format!(
                        "'{}' holds the value of secret '{secret}' revealed in this run",
                        file.path
                    ))
                }
            }
        }
        Ok((files, hashes))
    })
    .await?;

    let request = PushRequest {
        name: &artifact.name,
        files: files
            .iter()
            .map(|file| PushedFile {
                path: &file.path,
                size: file.size,
                media_type: mime_guess::from_path(&file.path)
                    .first_or_octet_stream()
                    .essence_str()
                    .to_string(),
            })
            .collect(),
        entrypoint: artifact.entrypoint.as_deref(),
        access: &artifact.access,
        retention,
        lock: artifact.lock,
    };
    let pushed: Pushed = answer(api.post_json(api.run_url("artifacts"), &request).await?)?;

    let mut uploads = pushed
        .uploads
        .into_iter()
        .map(|file| (file.path, file.upload))
        .collect::<std::collections::HashMap<_, _>>();
    let transfers = files
        .into_iter()
        .zip(hashes)
        .map(|(file, sha256)| match uploads.remove(&file.path) {
            Some(upload) => Ok((file, sha256, upload)),
            None => Err(format!("Cadence.CI gave no upload for '{}'", file.path)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let completed = futures::stream::iter(transfers)
        .map(|(file, read_sha256, upload)| async move {
            let handle = std::fs::File::open(&file.full)
                .map_err(|err| format!("'{}' cannot be read: {err}", file.path))?;
            let (sha256, parts) =
                transfer::upload(Arc::new(std::sync::Mutex::new(handle)), file.size, &upload)
                    .await
                    .map_err(|problem| format!("'{}' {problem}", file.path))?;
            if sha256 != read_sha256 {
                return Err(format!("'{}' changed while it was pushed", file.path));
            }
            Ok(CompletedFile {
                path: file.path,
                sha256,
                parts,
            })
        })
        .buffer_unordered(PARALLEL_TRANSFERS)
        .try_collect::<Vec<_>>()
        .await?;

    let (status, body) = api
        .post_json(
            api.run_url(&format!("artifacts/{}/complete", pushed.artifact_id)),
            &ArtifactCompletion { files: completed },
        )
        .await?;
    if !status.is_success() {
        return Err(refusal(status, &body));
    }
    Ok((
        api.url(&format!("artifacts/{}/{}", api.run_id, artifact.name)),
        links,
    ))
}

#[cfg(feature = "mock")]
async fn pushing(
    _secrets: &SecretsAccess,
    _artifact: &Artifact,
) -> Result<(String, Vec<String>), String> {
    Err("artifacts cannot be pushed in this build".to_string())
}

/// Fetches the artifact `name` of the run `run` into `path` of `volume`, giving the number
/// of files written.
#[cfg(feature = "real")]
async fn fetching(volume: &str, path: &str, run: &str, name: &str) -> Result<usize, String> {
    use futures::StreamExt;

    check_name(name)?;
    let run = uuid::Uuid::parse_str(run).map_err(|_| format!("'{run}' is not a run identifier"))?;
    let api = RunApi::from_environment().map_err(|problem| {
        format!("artifacts are only fetched by programs run by Cadence.CI, and {problem}")
    })?;
    let target = volume_path(volume, path)?;

    let (status, body) = api
        .get(format!(
            "{}?manifest",
            api.url(&format!("artifacts/{run}/{name}"))
        ))
        .await?;
    if status == reqwest::StatusCode::GONE {
        return Err("it expired".to_string());
    }
    let manifest: Manifest = answer((status, body))?;
    for file in &manifest.files {
        tree::check_path(&file.path)?;
        if file.sha256.is_none() {
            return Err(format!("'{}' has no sha256 to check it against", file.path));
        }
    }

    let created = create_directories(&target, &manifest.files)?;
    let downloads = futures::stream::iter(manifest.files)
        .map(|file| {
            let destination = file
                .path
                .split('/')
                .fold(target.clone(), |path, part| path.join(part));
            async move {
                let directory = destination.parent().unwrap_or(&destination).to_path_buf();
                let temporary = tempfile::NamedTempFile::new_in(directory)
                    .map_err(|err| format!("'{}' cannot be written: {err}", file.path))?;
                let handle = temporary
                    .reopen()
                    .map_err(|err| format!("'{}' cannot be written: {err}", file.path))?;
                transfer::download(
                    &file.download,
                    file.size,
                    file.sha256.as_deref().unwrap_or_default(),
                    handle,
                )
                .await
                .map_err(|problem| format!("'{}' not fetched, {problem}", file.path))?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(
                        temporary.path(),
                        std::fs::Permissions::from_mode(0o644),
                    )
                    .map_err(|err| format!("'{}' cannot be written: {err}", file.path))?;
                }
                Ok::<_, String>((temporary, destination))
            }
        })
        .buffer_unordered(PARALLEL_TRANSFERS)
        .collect::<Vec<_>>()
        .await;
    // Temporary files left are removed as the collection ends.
    let downloads: Result<Vec<_>, String> = downloads.into_iter().collect();
    let downloads = match downloads {
        Ok(downloads) => downloads,
        Err(problem) => {
            remove_directories(&created);
            return Err(problem);
        }
    };

    let count = downloads.len();
    let mut pending = downloads.into_iter();
    let mut placed = Vec::new();
    while let Some((temporary, destination)) = pending.next() {
        match temporary.persist(&destination) {
            Ok(_) => placed.push(destination),
            Err(err) => {
                let problem = format!(
                    "'{}' cannot be written: {}",
                    destination.display(),
                    err.error
                );
                drop(err);
                drop(pending);
                for file in placed {
                    let _ = std::fs::remove_file(file);
                }
                remove_directories(&created);
                return Err(problem);
            }
        }
    }
    Ok(count)
}

#[cfg(feature = "mock")]
async fn fetching(_volume: &str, _path: &str, _run: &str, _name: &str) -> Result<usize, String> {
    Err("artifacts cannot be fetched in this build".to_string())
}

/// Creates `target` and the directories of `files` within it, giving those created, in the
/// order they were, and refusing directories leading out of `target` through links.
#[cfg(feature = "real")]
fn create_directories(
    target: &std::path::Path,
    files: &[ManifestFile],
) -> Result<Vec<std::path::PathBuf>, String> {
    let mut directories = target
        .ancestors()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|directory| directory.to_path_buf())
        .collect::<Vec<_>>();
    for file in files {
        let parts = file.path.split('/').collect::<Vec<_>>();
        let mut directory = target.to_path_buf();
        for part in &parts[..parts.len() - 1] {
            directory.push(part);
            directories.push(directory.clone());
        }
    }

    let mut created = Vec::new();
    for directory in directories {
        if std::fs::symlink_metadata(&directory).is_err() {
            if let Err(err) = std::fs::create_dir(&directory) {
                remove_directories(&created);
                return Err(format!(
                    "'{}' cannot be created: {err}",
                    directory.display()
                ));
            }
            created.push(directory);
        }
    }

    let reached = |directory: &std::path::Path| directory.canonicalize().ok();
    let root = reached(target);
    for file in files {
        let parent = file
            .path
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .unwrap_or("");
        let directory = parent
            .split('/')
            .filter(|part| !part.is_empty())
            .fold(target.to_path_buf(), |path, part| path.join(part));
        let within = match (&root, reached(&directory)) {
            (Some(root), Some(directory)) => directory.starts_with(root),
            _ => false,
        };
        if !within {
            remove_directories(&created);
            return Err(format!(
                "'{}' leads out of '{}' through a link",
                file.path,
                target.display()
            ));
        }
    }
    Ok(created)
}

/// Removes `directories`, created in that order, if they are empty.
#[cfg(feature = "real")]
fn remove_directories(directories: &[std::path::PathBuf]) {
    for directory in directories.iter().rev() {
        let _ = std::fs::remove_dir(directory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_checked() {
        for valid in ["app", "report.html", "build_x86-64.v2"] {
            assert!(check_name(valid).is_ok(), "{valid}");
        }
        for invalid in ["", ".", "..", "a/b", "a b", "été"] {
            assert!(check_name(invalid).is_err(), "{invalid}");
        }
    }

    /// Answers as `melodium-service-api` serializes them, and requests as it parses them.
    #[cfg(feature = "real")]
    #[test]
    fn exchanges_are_those_of_the_api() {
        let pushed = r#"{"artifact_id":"00000000-0000-0000-0000-000000000001","uploads":[{"path":"index.html","upload":{"single":{"request":{"method":"PUT","url":"https://s/f","headers":{"x-amz-meta":"v"}}}}}]}"#;
        let pushed: Pushed = serde_json::from_str(pushed).unwrap();
        assert_eq!(pushed.uploads[0].path, "index.html");
        assert!(matches!(pushed.uploads[0].upload, Upload::Single { .. }));

        let manifest = r#"{"artifact":{"id":"00000000-0000-0000-0000-000000000001","run_id":"00000000-0000-0000-0000-000000000001","organization_id":"00000000-0000-0000-0000-000000000001","name":"site","entrypoint":null,"access":"public","expire_date":null,"locked_date":null,"state":"complete","created_date":"1970-01-01T00:00:00Z","files":[],"copies":[]},"files":[{"path":"index.html","size":6,"sha256":"cd","media_type":"text/html","download":{"method":"GET","url":"https://s/f","headers":{"x-amz-meta":"v"}}}]}"#;
        let manifest: Manifest = serde_json::from_str(manifest).unwrap();
        assert_eq!(manifest.files[0].sha256.as_deref(), Some("cd"));
        assert_eq!(manifest.files[0].size, 6);

        let request = |retention, entrypoint| PushRequest {
            name: "site",
            files: vec![PushedFile {
                path: "index.html",
                size: 6,
                media_type: "text/html".to_string(),
            }],
            entrypoint,
            access: "public",
            retention,
            lock: false,
        };
        assert_eq!(
            serde_json::to_string(&request(None, None)).unwrap(),
            r#"{"name":"site","files":[{"path":"index.html","size":6,"media_type":"text/html"}],"access":"public","lock":false}"#
        );
        assert_eq!(
            serde_json::to_string(&request(Some(Retention::Days(30)), Some("index.html"))).unwrap(),
            r#"{"name":"site","files":[{"path":"index.html","size":6,"media_type":"text/html"}],"entrypoint":"index.html","access":"public","retention":{"days":30},"lock":false}"#
        );
        assert_eq!(
            serde_json::to_string(&request(Some(Retention::Never), None)).unwrap(),
            r#"{"name":"site","files":[{"path":"index.html","size":6,"media_type":"text/html"}],"access":"public","retention":"never","lock":false}"#
        );
        let completion = ArtifactCompletion {
            files: vec![CompletedFile {
                path: "index.html".to_string(),
                sha256: "cd".to_string(),
                parts: Vec::new(),
            }],
        };
        assert_eq!(
            serde_json::to_string(&completion).unwrap(),
            r#"{"files":[{"path":"index.html","sha256":"cd","parts":[]}]}"#
        );
    }
}
