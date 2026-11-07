
# Changelog

## [v0.11.1] (2026-11-06)

- Adding `work/artifact`: `pushArtifact` pushes the files of a directory as a run artifact of api.melodium.tech, each file in a track of its own (`std/flow/scatter`), read, scanned for revealed secret values with its path, and uploaded in parts while it is read, with its entrypoint (renders), access, retention and lock; `pushArtifactFile` pushes a single file under its name; `fetchArtifact` writes the files of an artifact of a run, downloaded with resumption and checked against their sha256, a file that differs being removed (#155).
- Adding `work/run::get` and `|api_root_url()`, reading endpoints of api.melodium.tech as the run, such as the stable addresses of artifacts (#155).
- Adding `work/cache`: `restoreCache` and `saveCache` restore and save a directory of any file system as a named cache of api.melodium.tech, assembled in Mélodium. Saves are archived (`fs/archive::tar`), scanned for revealed secret values (`std/secret::scan`), compressed (zstd) and uploaded in parts while they are made, as new versions, identical content keeping the current one. Restores are downloaded with resumption, checked against the recorded sha256, and extracted next to their target before replacing it, a miss being a warning, never an error (#154).
- Adding `work/run`: `|api_url()`, the address of the run on api.melodium.tech, the `RunApi` client carrying the token of the run in its secret headers, `post` calling an endpoint of the run, and `upload` uploading data through an upload session, its parts presigned and uploaded in tracks of their own (`std/flow/scatter::scatterGathering`), retried after cuts, then completed with their sha256 and size, or aborted (#154).
- Adding `secret::ApiRunSource`, the `api-run:token` secret source giving the token of the run, never plainly revealed, masked from the start of the engine (#154).
- `getFileSystem` gives the file system of a volume rooted at its directory (`fs_mel::rooted::RootedFileSystemEngine`), whatever the executors: `..`, absolute paths and links leading out of the volume are refused, including links that executors sharing the volume create meanwhile. `ContainerFileSystem` and `KubeFileSystem`, which joined paths to the volume, are removed.
- Adding `secret::ApiIdentitySource`, the `api-id:<audience>` secret source giving the OIDC identity token of the run, requested with `GET /execution/run/{id}/id-token?audience=<audience>`, kept until 30 seconds before it expires, wiped with the values of `api:`, and never plainly revealed (#153).
- Adding `work/secret::store` and `storeBytes`, storing a `Secret<string>` or `Secret<Vec<byte>>` on api.melodium.tech as a secret of the organization of the run, revealed only to send it, for later runs to read through `api:`; the value the engine kept for that name is forgotten (#152).
- Breaking: `|service_container` takes `secret_env`, after `env`, variables given as `Secret<string>`. Pull secrets and secret variables stored on api.melodium.tech (`api:<name>`) go by name in the worker request (`pull_secret_name`, `secret_env`), never revealed, other secrets by value when their transmission is `value`, and `local` ones are refused, which pull secrets with the default policy now are. Compose refuses secrets given by name (#151).
- Adding `secret::ApiSource`, the `api:<name>` secret source giving the secrets stored on api.melodium.tech to the run reading them, through `GET /execution/run/{id}/secret/{name}` with the token of the run; each value is read once, kept until the engine ends and wiped then and before exit, and never plainly revealed (#150).
- Giving Compose containers whose names are not valid hostnames, such as names with `_`, a hostname where the invalid characters become `-`, instead of failing.

## [v0.11.0] (2026-10-15)

- Moving from async-std, which is discontinued, to smol: the Mélodium API is requested with reqwest, like the reports to S3, and reqwest and Kubernetes run in a Tokio context given by async-compat, and an answer 409 to a run start, the run being already started, is no longer an error; files written in containers and Kubernetes volumes are flushed before `completed` (#104).
- Taking the `DistantEngine` API token as a secret, by default the `MELODIUM_API_TOKEN` environment variable, and container pull secrets as secrets (#136).
- Giving secret variables to commands run in containers (`--env NAME`) and Kubernetes pods (read by `/bin/sh` from standard input, which the image needs) (#136).
- Adding `|new_plain_access` for workers listening without TLS on loopback addresses (#139).
- Breaking: taking the keys of `|new_access` and `|new_plain_access` as secrets, revealed by `distrib::start` only to authenticate the connection. Malformed keys make the start fail instead of becoming the nil UUID. The keys of distant workers are secrets too, so `Access` values never show keys in logs nor debug events, and Compose debug output leaves them out. In Rust, `Access` holds its fields directly and `api::CommonAccess` is removed (#136).
- Giving Compose distribution engines their keys through the environment instead of their arguments, and allowing plain TCP to them over the network local to the host (#130).
- Adding the `system` parameter to `distant`, an operating system built with `|linux`, `|linux_distribution`, `|linux_distribution_version`, `|windows`, `|windows_version`, `|macos`, `|macos_version`, `|freebsd`, `|openbsd`, `|netbsd`, `|solaris`, `|illumos`, `|aix`, `|ibmi`, `|zos` (each with a `_version` variant), or one exact machine image built with `|image`; both cannot be requested together, and Compose only satisfies `linux` and `linux` with the `alpine` distribution (#141, #143).
- Adding every architecture of the OCI image specification, and `wasm32` and `wasm64`, to `work/resources/arch`; unknown architectures are now read as unknown instead of failing (#141).
- Reading the architecture names Rust uses (`x86_64`, `aarch64`, `i686`, `powerpc64le`, `loongarch64`, `mipsel`...) as aliases of the OCI ones in requests, architectures still being written with the OCI names, and adding a function of the same name to `work/resources/arch` for each (`|x86_64`, `|aarch64`, `|x86`, `|i686`, `|armv7`, `|riscv64gc`, `|powerpc64le`, `|powerpc64`, `|loongarch64`, `|mipsel`, `|mips64el`) (#141).
- Breaking: Compose refuses worker and container architectures different from the host one, instead of ignoring them (#141).
- Breaking: taking the architecture of `|container` and `|service_container` as optional, none meaning the one of the machine the worker runs on; programs wrap it with `|wrap<Arch>` (#141).
- Adding `|local_arch`, the architecture of the machine running the engine (#141).
- Breaking: `distant` sends no edition when the program gives none, meaning any edition suits, instead of `scratch`. The edition is the one of the engine (`standard`, `desktop`, `container`, ...), the image being defined by the distribution of `system`; Compose runs the `standard` and `container` editions on an Alpine image and refuses another edition. Documenting editions and tags in `distant` (#143).

## [v0.10.4] (2026-09-24)

- Decoupling `report_logs`/`report_debug` from the S3 upload path: uploading now runs in its own task, so a slow or repeatedly failing upload no longer blocks draining logs/debug events off the channel every running treatment feeds. A batch that fails to reach the reporting endpoint is dropped instead of retried inline.
- Bounding the queue of batches waiting to reach the reporting endpoint (`MELODIUM_REPORT_MAX_PENDING_BATCHES`, default 4): once full, newly completed batches are dropped instead of growing memory without limit. Batch size and interval are now also overridable via `MELODIUM_REPORT_BATCH_SIZE` and `MELODIUM_REPORT_BATCH_INTERVAL_SECS`.

## [v0.10.3] (2026-09-08)

- Adapting to `melodium-common`'s new panic-free value casting and packed-array APIs (no behavior change).

## [v0.10.2] (2026-08-04)

- Adding a detection timeout (overridable via `MELODIUM_COMPOSE_DETECTION_TIMEOUT_SECS`) so an unresponsive `podman`/`docker` daemon can no longer block `compose()`, and the whole distributed run, forever.
- Retrying the same batch instead of dropping it and advancing when sending logs or debug events to S3 fails.

## [v0.10.1] (2026-05-29)

- Adding program details to execution reporting (#106).

## [v0.10.0] (2026-03-02)

- Adding execution reporting (launch/end signals, chunk and data transmission).
- Adding execution group ID and run ID.
- Enabling WebAssembly compilation.
- Adding child process timeout.
- Turning connection timeout hard errors into soft ones.
- Suppressing pull progress output.

## [v0.9.2] (2026-01-15)

- Fixing SpawnTerminable behavior (#103).
- Adding network alias support in compose.
- Adding user definition in compose.

## [v0.9.0]

First release.

