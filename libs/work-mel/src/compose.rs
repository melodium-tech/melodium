use crate::api::{Access, /*Arch,*/ ModeRequest, Request};
#[cfg(feature = "real")]
use async_std::{
    net::ToSocketAddrs,
    process::{Child, Command},
    task::sleep,
};
use compose_spec::{
    service::{
        ports::{Protocol, Range, ShortPort, ShortRanges},
        volumes::{
            self,
            mount::{Bind, BindOptions, Common, Volume},
            HostPath, Mount,
        },
        AbsolutePath, /*Cpus,*/ Image,
    },
    Compose, Identifier, ListOrMap, Map, MapKey, Service, /*Value,*/
};
use core::{
    net::{Ipv4Addr /*Ipv6Addr, SocketAddr*/},
    time::Duration,
};
use futures::AsyncWriteExt;
use std::{fmt::Display, process::Stdio, sync::OnceLock};
use uuid::Uuid;

static IMAGES_PULL_SOURCE: OnceLock<String> = OnceLock::new();

#[derive(Debug, PartialEq, Eq)]
pub enum Executor {
    Podman,
    Docker,
}

impl Display for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Executor::Podman => write!(f, "podman"),
            Executor::Docker => write!(f, "docker"),
        }
    }
}

#[cfg(feature = "real")]
pub async fn compose(mut request: Request) -> Result<(Access, Child), Vec<String>> {
    if request.edition.as_str() == "scratch" {
        request.edition = "alpine".to_string()
    }

    let enable_debug = std::env::var("MELODIUM_COMPOSE_DEBUG")
        .map(|val| val == "true")
        .unwrap_or(false);

    /*if !request
        .arch
        .map(|arch| match env!("ARCH") {
            "x86_64" => arch == Arch::Amd64,
            "aarch64" => arch == Arch::Arm64,
            _ => false,
        })
        .unwrap_or(false)
    {
        return Err(vec![
            "Host architecture does not match requirements".to_string()
        ]);
    }*/

    let executor = if let Ok(_output) = Command::new("podman").args(&["version"]).output().await {
        Executor::Podman
    } else if let Ok(_output) = Command::new("docker").args(&["version"]).output().await {
        Executor::Docker
    } else {
        return Err(vec!["No executor available".to_string()]);
    };

    let socket = if let Ok(output) = Command::new(executor.to_string())
        .args(&["info", "--format", "{{ .Host.RemoteSocket.Path }}"])
        .output()
        .await
    {
        if output.status.success() {
            Some(
                String::from_utf8(output.stdout)
                    .map(|out| out.trim().to_string())
                    .map_err(|err| vec![err.to_string()])?,
            )
        } else {
            None
        }
    } else {
        return Err(vec!["No socket available with".to_string()]);
    };

    if enable_debug {
        eprintln!("Socket: {socket:?}");
    }

    let id = Uuid::new_v4();
    let short_id = format!("{id:.*}", 8);

    let access_key = Uuid::new_v4();

    let volumes = request
        .volumes
        .iter()
        .map(|volume| {
            (
                Identifier::new(format!("volume-custom-{}", volume.name)).unwrap(),
                None,
            )
        })
        .collect();

    let mut containers = Vec::new();
    for container in &request.containers {
        let mut mounts = volumes::Volumes::new();
        for mount in &container.mounts {
            mounts.insert(
                Mount::Volume(Volume {
                    source: Some(
                        Identifier::new(format!("volume-custom-{}", mount.name))
                            .map_err(|err| vec![err.to_string()])?,
                    ),
                    common: Common {
                        target: AbsolutePath::new(mount.mount_point.clone())
                            .map_err(|err| vec![err.to_string()])
                            .unwrap(),
                        read_only: false,
                        consistency: None,
                        extensions: [].into(),
                    },
                    volume: None,
                })
                .into(),
            );
        }

        let service = Service {
            container_name: Some(
                Identifier::new(format!("{short_id}-container-custom-{}", container.name))
                    .map_err(|err| vec![err.to_string()])?,
            ),
            image: Some(
                Image::parse(container.image.clone()).map_err(|err| vec![err.to_string()])?,
            ),
            command: Some(compose_spec::service::Command::List(vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "trap : TERM INT; sleep 9999999999d & wait".to_string(),
            ])),
            /*cpus: Some(
                Cpus::new(container.cpu as f64 / 1000f64).map_err(|err| vec![err.to_string()])?,
            ),*/
            /*mem_limit: Some(compose_spec::service::ByteValue::Megabytes(
                container.memory as u64,
            )),*/
            /*storage_opt: [(
                MapKey::new("size").map_err(|err| vec![err.to_string()])?,
                Some(Value::parse(format!("{}M", container.storage))),
            )]
            .into(),*/
            volumes: mounts,
            ..Default::default()
        };
        containers.push(service);
    }

    for container in &request.service_containers {
        let mut mounts = volumes::Volumes::new();
        for mount in &container.mounts {
            mounts.insert(
                Mount::Volume(Volume {
                    source: Some(
                        Identifier::new(format!("volume-custom-{}", mount.name))
                            .map_err(|err| vec![err.to_string()])?,
                    ),
                    common: Common {
                        target: AbsolutePath::new(mount.mount_point.clone())
                            .map_err(|err| vec![err.to_string()])
                            .unwrap(),
                        read_only: false,
                        consistency: None,
                        extensions: [].into(),
                    },
                    volume: None,
                })
                .into(),
            );
        }

        let service = Service {
            container_name: Some(
                Identifier::new(format!("{short_id}-container-service-{}", container.name))
                    .map_err(|err| vec![err.to_string()])?,
            ),
            image: Some(
                Image::parse(container.image.clone()).map_err(|err| vec![err.to_string()])?,
            ),
            environment: ListOrMap::Map(
                container
                    .env
                    .iter()
                    .map(|(name, value)| {
                        (
                            MapKey::new(name.clone()).unwrap(),
                            Some(value.clone().into()),
                        )
                    })
                    .collect(),
            ),
            command: container
                .command
                .as_ref()
                .map(|command| compose_spec::service::Command::List(command.clone())),
            /*cpus: Some(
                Cpus::new(container.cpu as f64 / 1000f64).map_err(|err| vec![err.to_string()])?,
            ),*/
            /*mem_limit: Some(compose_spec::service::ByteValue::Megabytes(
                container.memory as u64,
            )),*/
            /*storage_opt: [(
                MapKey::new("size").map_err(|err| vec![err.to_string()])?,
                Some(Value::parse(format!("{}M", container.storage))),
            )]
            .into(),*/
            volumes: mounts,
            ..Default::default()
        };
        containers.push(service);
    }

    let mut mounts = volumes::Volumes::new();
    if let Some(socket) = socket {
        mounts.insert(
            Mount::Bind(Bind {
                source: HostPath::new(socket).map_err(|err| vec![err.to_string()])?,
                common: Common {
                    target: AbsolutePath::new(match executor {
                        Executor::Podman => "/run/podman/podman.sock",
                        Executor::Docker => "/var/run/docker.sock",
                    })
                    .map_err(|err| vec![err.to_string()])?,
                    read_only: false,
                    consistency: None,
                    extensions: [].into(),
                },
                bind: Some(BindOptions {
                    selinux: Some(volumes::SELinux::Shared),
                    ..Default::default()
                }),
            })
            .into(),
        );
    } else if executor == Executor::Docker {
        mounts.insert(
            Mount::Bind(Bind {
                source: HostPath::new("/var/run/docker.sock")
                    .map_err(|err| vec![err.to_string()])?,
                common: Common {
                    target: AbsolutePath::new("/var/run/docker.sock")
                        .map_err(|err| vec![err.to_string()])?,
                    read_only: false,
                    consistency: None,
                    extensions: [].into(),
                },
                bind: Some(BindOptions {
                    selinux: Some(volumes::SELinux::Shared),
                    ..Default::default()
                }),
            })
            .into(),
        );
    }
    let bind_ip = if executor == Executor::Docker {
        if let Ok(mut socket_iter) = ("docker", 0).to_socket_addrs().await {
            if let Some(socket) = socket_iter.next() {
                if enable_debug {
                    eprintln!("Docker socket: {socket:?}");
                }
                socket.ip()
            } else {
                Ipv4Addr::LOCALHOST.into()
            }
        } else {
            Ipv4Addr::LOCALHOST.into()
        }
    } else {
        Ipv4Addr::LOCALHOST.into()
    };

    for volume in &request.volumes {
        mounts.insert(
            Mount::Volume(Volume {
                source: Some(Identifier::new(format!("volume-custom-{}", volume.name)).unwrap()),
                common: Common {
                    target: AbsolutePath::new(format!("/media/{}", volume.name))
                        .map_err(|err| vec![err.to_string()])?,
                    read_only: false,
                    consistency: None,
                    extensions: [].into(),
                },
                volume: None,
            })
            .into(),
        );
    }

    let mut environment = Map::new();
    environment.insert(
        MapKey::new("MELODIUM_JOB_EXECUTOR").map_err(|err| vec![err.to_string()])?,
        Some(executor.to_string().into()),
    );
    environment.insert(
        MapKey::new("MELODIUM_JOB_CONTAINERS").map_err(|err| vec![err.to_string()])?,
        Some(
            request
                .containers
                .iter()
                .map(|container| container.name.clone())
                .collect::<Vec<_>>()
                .join(",")
                .into(),
        ),
    );
    environment.insert(
        MapKey::new("MELODIUM_JOB_SERVICE_CONTAINERS").map_err(|err| vec![err.to_string()])?,
        Some(
            request
                .service_containers
                .iter()
                .map(|container| container.name.clone())
                .collect::<Vec<_>>()
                .join(",")
                .into(),
        ),
    );
    environment.insert(
        MapKey::new("MELODIUM_JOB_VOLUMES").map_err(|err| vec![err.to_string()])?,
        Some(
            request
                .volumes
                .iter()
                .map(|volume| volume.name.clone())
                .collect::<Vec<_>>()
                .join(",")
                .into(),
        ),
    );

    for container in &request.containers {
        environment.insert(
            MapKey::new(format!("MELODIUM_JOB_CONTAINER_{}", container.name))
                .map_err(|err| vec![err.to_string()])?,
            Some(format!("{short_id}-container-custom-{}", container.name).into()),
        );
    }
    for container in &request.service_containers {
        environment.insert(
            MapKey::new(format!("MELODIUM_JOB_SERVICE_CONTAINER_{}", container.name))
                .map_err(|err| vec![err.to_string()])?,
            Some(format!("{short_id}-container-service-{}", container.name).into()),
        );
    }
    for volume in &request.volumes {
        environment.insert(
            MapKey::new(format!("MELODIUM_JOB_VOLUME_{}", volume.name))
                .map_err(|err| vec![err.to_string()])?,
            Some(format!("/media/{}", volume.name).into()),
        );
    }

    let melodium_service_name = format!("{short_id}-melodium");
    let melodium = Service {
        container_name: Some(Identifier::new(melodium_service_name.as_str()).unwrap()),
        image: Some(
            Image::parse(format!(
                "{}/melodium:{}-{}-{}",
                IMAGES_PULL_SOURCE.get_or_init(|| {
                    std::env::var("MELODIUM_IMAGES_PULL_SOURCE")
                        .unwrap_or("quay.io/melodium".to_string())
                }),
                request.version,
                request.edition,
                executor
            ))
            .map_err(|err| vec![err.to_string()])?,
        ),
        depends_on: compose_spec::ShortOrLong::Short(
            containers
                .iter()
                .filter_map(|container| container.container_name.clone())
                .collect(),
        ),
        environment: compose_spec::ListOrMap::Map(environment),
        command: Some(compose_spec::service::Command::List(match &request.mode {
            ModeRequest::Direct {
                entrypoint,
                project: _,
            } => vec!["run".to_string(), entrypoint.clone()],
            ModeRequest::Distribute { key } => vec![
                "dist".to_string(),
                "--ip".to_string(),
                "0.0.0.0".to_string(),
                "--port".to_string(),
                "8080".to_string(),
                "--wait".to_string(),
                "30".to_string(),
                "--duration".to_string(),
                request.max_duration.to_string(),
                "--recv-key".to_string(),
                access_key.to_string(),
                "--send-key".to_string(),
                key.to_string(),
                if bind_ip == Ipv4Addr::LOCALHOST {
                    "--localhost".to_string()
                } else {
                    "--disable-tls".to_string()
                },
            ],
        })),
        //cpus: Some(Cpus::new(request.cpu as f64 / 1000f64).map_err(|err| vec![err.to_string()])?),
        /*mem_limit: Some(compose_spec::service::ByteValue::Megabytes(
            request.memory as u64,
        )),*/
        /*storage_opt: [(
            MapKey::new("size").unwrap(),
            Some(Value::parse(format!("{}M", request.storage))),
        )]
        .into(),*/
        ports: [ShortPort {
            host_ip: Some(Ipv4Addr::UNSPECIFIED.into()),
            ranges: ShortRanges::new(
                None,
                Range::new(8080, None).map_err(|err| vec![err.to_string()])?,
            )
            .map_err(|err| vec![err.to_string()])?,
            protocol: Some(Protocol::Tcp),
        }
        .into()]
        .into(),
        volumes: mounts,
        ..Default::default()
    };

    containers.push(melodium);

    let compose_spec = Compose {
        services: containers
            .into_iter()
            .map(|container| (container.container_name.clone().unwrap(), container))
            .collect(),
        volumes: volumes,
        ..Default::default()
    };

    if let Some(path) = std::env::var_os("MELODIUM_COMPOSE_DUMP") {
        let _ = std::fs::write(
            path,
            serde_yaml::to_string(&compose_spec).unwrap().as_bytes(),
        );
    }

    match Command::new(executor.to_string())
        .args(&[
            "compose",
            "-f",
            "-",
            "up",
            "--abort-on-container-exit",
            "--no-color",
            "--force-recreate",
            "--exit-code-from",
            melodium_service_name.as_str(),
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(stdin) = child.stdin.take() {
                {
                    let mut stdin = stdin;
                    stdin
                        .write_all(
                            serde_yaml::to_string(&compose_spec)
                                .map_err(|err| vec![err.to_string()])?
                                .as_bytes(),
                        )
                        .await
                        .map_err(|err| vec![err.to_string()])?;
                    let _ = stdin.close().await;
                }

                let mut success = false;
                let mut timeout = 0;
                while child
                    .try_status()
                    .map_err(|err| vec![err.to_string()])?
                    .is_none()
                {
                    if let Ok(output) = Command::new(executor.to_string())
                        .args(&[
                            "inspect",
                            "--format",
                            "{{ .State.Running }}",
                            melodium_service_name.as_str(),
                        ])
                        .output()
                        .await
                    {
                        let status = String::from_utf8_lossy(output.stdout.as_slice())
                            .trim()
                            .to_string();
                        if enable_debug {
                            eprintln!(
                                "Status: {:?}",
                                String::from_utf8_lossy(output.stdout.as_slice())
                            );
                        }
                        if status.as_str() == "true" {
                            success = true;
                            break;
                        }
                    }
                    sleep(Duration::from_secs(1)).await;
                    timeout += 1;

                    // Do not wait more than 10 minutes to launch
                    if timeout > 600 {
                        break;
                    }
                }

                if success {
                    let binding = match Command::new(executor.to_string())
                        .args(&["port", melodium_service_name.as_str(), "8080/tcp"])
                        .output()
                        .await
                    {
                        Ok(output) if output.status.success() => {
                            if enable_debug {
                                eprintln!("Port: {}", String::from_utf8_lossy(&output.stdout));
                            }
                            let port = String::from_utf8_lossy(&output.stdout)
                                .split_once(':')
                                .ok_or_else(|| vec!["Unable to get exposed port".to_string()])?
                                .1
                                .trim()
                                .to_string();
                            port.parse::<u16>().map_err(|err| vec![err.to_string()])?
                        }
                        Ok(output) => {
                            return Err(vec![String::from_utf8_lossy(&output.stderr).to_string()])
                        }
                        Err(err) => return Err(vec![err.to_string()]),
                    };

                    if enable_debug {
                        let _ = Command::new(executor.to_string())
                            .args(["info"])
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .output()
                            .await;
                        let _ = Command::new(executor.to_string())
                            .args(["container", "ls", "-a"])
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .output()
                            .await;
                        let _ = Command::new(executor.to_string())
                            .args(["network", "ls"])
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .output()
                            .await;
                    }

                    let access = Access {
                        id: id,
                        addresses: vec![bind_ip],
                        port: binding,
                        key: access_key,
                        disable_tls: bind_ip != Ipv4Addr::LOCALHOST,
                    };

                    if enable_debug {
                        eprintln!("Access: {access:#?}");
                    }

                    Ok((access, child))
                } else {
                    let _ = child.kill();
                    match child.output().await {
                        Ok(output) => Err(vec![
                            String::from_utf8_lossy(&output.stdout).to_string(),
                            String::from_utf8_lossy(&output.stderr).to_string(),
                            format!("Executor '{}' exit code: {}", executor, output.status),
                        ]),
                        Err(err) => Err(vec![err.to_string()]),
                    }
                }
            } else {
                Err(vec!["Unable to get executor stdin".to_string()])
            }
        }
        Err(err) => Err(vec![err.to_string()]),
    }
}
