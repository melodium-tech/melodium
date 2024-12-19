pub mod arch;

use crate::api;
use arch::*;
use fs_mel::filesystem::*;
use melodium_core::*;
use melodium_macro::{mel_data, mel_function, mel_treatment};
use process_mel::exec::*;

#[mel_treatment(
    input trigger Block<void>
    output executor Block<Executor>
    output error Block<string>
    output failed Block<void>
)]
pub async fn getExecutor(name: string) {
    if let Ok(_) = trigger.recv_one().await {
        #[cfg(not(feature = "kubernetes"))]
        {
            let _ = failed.send_one(().into()).await;
            let _ = error
                .send_one("No executor available".to_string().into())
                .await;
        }

        #[cfg(feature = "kubernetes")]
        {
            match crate::kube::KubeExecutor::try_new(name).await {
                Ok(kube_exec) => {
                    let _ = executor
                        .send_one(
                            (std::sync::Arc::new(Executor {
                                executor: std::sync::Arc::new(kube_exec),
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
    }
}

#[mel_treatment(
    input trigger Block<void>
    output filesystem Block<FileSystem>
    output error Block<string>
    output failed Block<void>
)]
pub async fn getFileSystem(name: string) {
    if let Ok(_) = trigger.recv_one().await {
        #[cfg(not(feature = "kubernetes"))]
        {
            let _ = failed.send_one(().into()).await;
            let _ = error
                .send_one("No filesystem available".to_string().into())
                .await;
        }

        #[cfg(feature = "kubernetes")]
        {
            match crate::kube::KubeFileSystem::try_new(name).await {
                Ok(kube_fs) => {
                    let _ = filesystem
                        .send_one(
                            (std::sync::Arc::new(FileSystem {
                                filesystem: std::sync::Arc::new(kube_fs),
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
