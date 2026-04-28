pub mod arch;

use crate::api;
use arch::*;
use fs_mel::filesystem::*;
use melodium_core::*;
use melodium_macro::{mel_data, mel_function, mel_treatment};
use process_mel::exec::*;
use std_mel::data::string_map::*;

/// Retrieve an executor by name from the current work environment.
///
/// `name` identifies the executor (container or Kubernetes pod) to look up.
/// When `name` is `None`, a local executor is returned directly.
///
/// `executor` is emitted when the executor is found and ready.
/// `failed` and `error` are emitted if the executor cannot be located or initialised.
///
/// ```mermaid
/// graph LR
///     T("getExecutor()")
///     N["〈🟦〉"] -->|name| T
///     T -->|executor| E["〈🟩〉"]
///     T -->|error| ER["〈🟫〉"]
///     T -->|failed| F["〈🟥〉"]
///     style N fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
///     style ER fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input name Block<Option<string>>
    output executor Block<Executor>
    output error Block<string>
    output failed Block<void>
)]
pub async fn getExecutor() {
    #[cfg(feature = "real")]
    if let Ok(name) = name
        .recv_one()
        .await
        .map(|val| GetData::<Option<String>>::try_data(val).unwrap())
    {
        if let Some(name) = name {
            match std::env::var("MELODIUM_RUN_EXECUTOR").as_deref() {
                Ok("podman") | Ok("docker") => {
                    match crate::container::ContainerExecutor::try_new(name).await {
                        Ok(container_exec) => {
                            let _ = executor
                                .send_one(
                                    (std::sync::Arc::new(Executor {
                                        executor: std::sync::Arc::new(container_exec),
                                    })
                                        as std::sync::Arc<dyn Data>)
                                        .into(),
                                )
                                .await;
                        }
                        Err(err) => {
                            let _ = failed.send_one(().into()).await;
                            let _ = error.send_one(err.into()).await;
                        }
                    }
                }
                _ => {
                    #[cfg(feature = "kubernetes")]
                    {
                        match crate::kube::KubeExecutor::try_new(name).await {
                            Ok(kube_exec) => {
                                let _ = executor
                                    .send_one(
                                        (std::sync::Arc::new(Executor {
                                            executor: std::sync::Arc::new(kube_exec),
                                        })
                                            as std::sync::Arc<dyn Data>)
                                            .into(),
                                    )
                                    .await;
                            }
                            Err(err) => {
                                let _ = failed.send_one(().into()).await;
                                let _ = error.send_one(err.into()).await;
                            }
                        }
                    }
                    #[cfg(not(feature = "kubernetes"))]
                    {
                        let _ = failed.send_one(().into()).await;
                        let _ = error
                            .send_one("Executor name not set".to_string().into())
                            .await;
                    }
                }
            }
        } else {
            let _ = executor
                .send_one(
                    (std::sync::Arc::new(Executor {
                        executor: std::sync::Arc::new(process_mel::local::LocalExecutorEngine {}),
                    }) as std::sync::Arc<dyn Data>)
                        .into(),
                )
                .await;
        }
    }
    #[cfg(feature = "mock")]
    {
        let _ = failed.send_one(().into()).await;
        let _ = error.send_one("Mock mode".to_string().into()).await;
    }
}

/// Retrieve a filesystem handle by volume name from the current work environment.
///
/// `name` identifies the volume to look up (container volume or Kubernetes persistent volume).
///
/// `filesystem` is emitted when the volume is found and ready.
/// `failed` and `error` are emitted if the volume cannot be located.
///
/// ```mermaid
/// graph LR
///     T("getFileSystem()")
///     N["〈🟦〉"] -->|name| T
///     T -->|filesystem| FS["〈🟩〉"]
///     T -->|error| ER["〈🟫〉"]
///     T -->|failed| F["〈🟥〉"]
///     style N fill:#ffff,stroke:#ffff
///     style FS fill:#ffff,stroke:#ffff
///     style ER fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input name Block<string>
    output filesystem Block<FileSystem>
    output error Block<string>
    output failed Block<void>
)]
pub async fn getFileSystem() {
    #[cfg(feature = "real")]
    if let Ok(name) = name
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        match std::env::var("MELODIUM_RUN_EXECUTOR").as_deref() {
            Ok("podman") | Ok("docker") => {
                match crate::container::ContainerFileSystem::try_new(name).await {
                    Ok(container_fs) => {
                        let _ = filesystem
                            .send_one(
                                (std::sync::Arc::new(FileSystem {
                                    filesystem: std::sync::Arc::new(container_fs),
                                }) as std::sync::Arc<dyn Data>)
                                    .into(),
                            )
                            .await;
                    }
                    Err(err) => {
                        let _ = failed.send_one(().into()).await;
                        let _ = error.send_one(err.into()).await;
                    }
                }
            }
            _ => {
                #[cfg(feature = "kubernetes")]
                {
                    match crate::kube::KubeFileSystem::try_new(name).await {
                        Ok(kube_fs) => {
                            let _ = filesystem
                                .send_one(
                                    (std::sync::Arc::new(FileSystem {
                                        filesystem: std::sync::Arc::new(kube_fs),
                                    })
                                        as std::sync::Arc<dyn Data>)
                                        .into(),
                                )
                                .await;
                        }
                        Err(err) => {
                            let _ = failed.send_one(().into()).await;
                            let _ = error.send_one(err.into()).await;
                        }
                    }
                }
                #[cfg(not(feature = "kubernetes"))]
                {
                    let _ = failed.send_one(().into()).await;
                    let _ = error
                        .send_one(format!("No volume '{name}' available").into())
                        .await;
                }
            }
        }
    }
    #[cfg(feature = "mock")]
    {
        let _ = failed.send_one(().into()).await;
        let _ = error.send_one("Mock mode".to_string().into()).await;
    }
}

/// Container specification for a work request.
///
/// Used in `distant` to request containers that run alongside the Mélodium engine as executors.
#[mel_data]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Container(pub api::Container);

/// Build a `Container` specification.
///
/// - `name`: unique name for this container within the request.
/// - `memory`: memory allocated in megabytes.
/// - `cpu`: CPU allocated in millicores.
/// - `storage`: ephemeral storage in megabytes.
/// - `arch`: CPU architecture required.
/// - `mounts`: list of volume mounts.
/// - `image`: container image reference.
/// - `pull_secret`: optional image pull secret (defaults to `"{}"`).
#[mel_function]
pub fn container(
    name: string,
    memory: u32,
    cpu: u32,
    storage: u32,
    arch: Arch,
    mounts: Vec<Mount>,
    image: string,
    pull_secret: Option<string>,
) -> Container {
    Container(api::Container {
        name,
        image,
        pull_secret: pull_secret.unwrap_or_else(|| "{}".to_string()),
        memory,
        cpu,
        storage,
        arch: arch.0,
        mounts: mounts.into_iter().map(|mount| mount.0).collect(),
    })
}

/// Service container specification for a work request.
///
/// Service containers run alongside the Mélodium engine and are accessible as network services,
/// but are not directly used as executors.
#[mel_data]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceContainer(pub api::ServiceContainer);

/// Build a `ServiceContainer` specification.
///
/// - `name`: unique name for this service container.
/// - `memory`, `cpu`, `storage`, `arch`, `mounts`, `image`, `pull_secret`: same as `|container`.
/// - `env`: optional environment variables for the container.
/// - `command`: optional entrypoint override.
#[mel_function]
pub fn service_container(
    name: string,
    memory: u32,
    cpu: u32,
    storage: u32,
    arch: Arch,
    mounts: Vec<Mount>,
    image: string,
    pull_secret: Option<string>,
    env: Option<StringMap>,
    command: Option<Vec<string>>,
) -> ServiceContainer {
    ServiceContainer(api::ServiceContainer {
        name,
        image,
        pull_secret: pull_secret.unwrap_or_else(|| "{}".to_string()),
        memory,
        cpu,
        storage,
        arch: arch.0,
        mounts: mounts.into_iter().map(|mount| mount.0).collect(),
        env: env.map(|map| map.map).unwrap_or_default(),
        command: command,
    })
}

/// A volume mount point that maps a named volume into a container.
#[mel_data]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mount(pub api::VolumeMount);

/// Build a `Mount` that maps `name` to `point` inside a container.
#[mel_function]
pub fn mount(name: string, point: string) -> Mount {
    Mount(api::VolumeMount {
        name,
        mount_point: point,
    })
}

/// A shared filesystem volume for work requests.
#[mel_data]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Volume(pub api::Volume);

/// Build a `Volume` with the given `name` and `storage` size in megabytes.
#[mel_function]
pub fn volume(name: string, storage: u32) -> Volume {
    Volume(api::Volume { name, storage })
}
