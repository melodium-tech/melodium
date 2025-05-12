pub mod arch;

use crate::api;
use arch::*;
use fs_mel::filesystem::*;
use melodium_core::*;
use melodium_macro::{mel_data, mel_function, mel_treatment};
use process_mel::exec::*;
use std_mel::data::string_map::*;

#[mel_treatment(
    input name Block<string>
    output executor Block<Executor>
    output error Block<string>
    output failed Block<void>
)]
pub async fn getExecutor() {
    #[allow(unused)]
    if let Ok(name) = name
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        match std::env::var("MELODIUM_JOB_EXECUTOR").as_deref() {
            Ok("podman") | Ok("docker") => {
                match crate::container::ContainerExecutor::try_new(name).await {
                    Ok(container_exec) => {
                        let _ = executor
                            .send_one(
                                (std::sync::Arc::new(Executor {
                                    executor: std::sync::Arc::new(container_exec),
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
    }
}

#[mel_treatment(
    input name Block<string>
    output filesystem Block<FileSystem>
    output error Block<string>
    output failed Block<void>
)]
pub async fn getFileSystem() {
    #[allow(unused)]
    if let Ok(name) = name
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        match std::env::var("MELODIUM_JOB_EXECUTOR").as_deref() {
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
}

#[mel_data]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Container(pub api::Container);

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

#[mel_data]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceContainer(pub api::ServiceContainer);

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

#[mel_data]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mount(pub api::VolumeMount);

#[mel_function]
pub fn mount(name: string, point: string) -> Mount {
    Mount(api::VolumeMount {
        name,
        mount_point: point,
    })
}

#[mel_data]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Volume(pub api::Volume);

#[mel_function]
pub fn volume(name: string, storage: u32) -> Volume {
    Volume(api::Volume { name, storage })
}
