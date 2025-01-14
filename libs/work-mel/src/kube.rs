use async_std::{
    fs::{self, DirBuilder, OpenOptions},
    io::{ReadExt, WriteExt},
    path::{Path, PathBuf},
    stream::StreamExt,
};
use async_trait::async_trait;
use async_walkdir::{Filtering, WalkDir};
use core::fmt::Debug;
use fs_mel::filesystem::FileSystemEngine;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::AttachParams, Api, Client};
use melodium_core::common::executive::*;
use melodium_macro::check;
use process_mel::{command::Command, environment::Environment, exec::*};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

pub struct KubeExecutor {
    client: Client,
    pod: String,
    container: String,
    container_full_name: String,
}

impl KubeExecutor {
    pub async fn try_new(container: String) -> Result<KubeExecutor, String> {
        if Ok(true)
            != std::env::var("MELODIUM_JOB_CONTAINERS").map(|var| {
                var.split(",")
                    .any(|var_container| var_container == container)
            })
        {
            return Err(format!("No container '{container}' listed as available"));
        }

        if let Ok(container_full_name) =
            std::env::var(format!("MELODIUM_JOB_CONTAINER_{container}"))
        {
            if let Ok(pod) = std::env::var("MELODIUM_POD_NAME") {
                Client::try_default()
                    .await
                    .map(|client| Self {
                        client,
                        pod,
                        container,
                        container_full_name,
                    })
                    .map_err(|err| format!("No kubernetes access available: {err}"))
            } else {
                return Err(format!("No pod name available"));
            }
        } else {
            return Err(format!("No container '{container}' available"));
        }
    }
}

#[async_trait]
impl ExecutorEngine for KubeExecutor {
    async fn exec(
        &self,
        command: &Command,
        environment: Option<&Environment>,
        started: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
        exit: OnceCodeCall<'async_trait>,
    ) {
        let pod: Api<Pod> = Api::namespaced(self.client.clone(), "melodium");

        let mut full_command = if let Some(environment) = environment {
            let mut env_command = vec!["/usr/bin/env".to_string()];

            if environment.clear_env {
                env_command.push("--ignore-environment".to_string());
            }

            if let Some(dir) = &environment.working_directory {
                env_command.push(format!("--chdir={dir}"));
            }

            for (name, val) in &environment.variables.map {
                env_command.push(format!(
                    "{name}={}",
                    if val
                        .datatype()
                        .implements(&melodium_core::common::descriptor::DataTrait::ToString)
                    {
                        melodium_core::DataTrait::to_string(val)
                    } else {
                        "".to_string()
                    }
                ));
            }

            env_command.push("--split-string".to_string());

            env_command.push(command.command.clone());

            env_command
        } else {
            vec![command.command.clone()]
        };

        full_command.extend(command.arguments.clone());

        match pod
            .exec(
                &self.pod,
                full_command,
                &AttachParams::default().container(self.container_full_name.clone()),
            )
            .await
        {
            Ok(mut process) => {
                if let Some(status_waiter) = process.take_status() {
                    started().await;

                    if let Some(status) = status_waiter.await {
                        match status.status.as_ref().map(|s| s.as_str()) {
                            Some("Success") => {
                                completed().await;
                                exit(status.code).await;
                            }
                            _ => {
                                failed().await;
                                error(
                                    status
                                        .reason
                                        .unwrap_or_else(|| "No reason provided".to_string()),
                                )
                                .await;
                            }
                        }
                    } else {
                        failed().await;
                        error("No output status provided".to_string()).await;
                    }
                } else {
                    failed().await;
                    error("Unable to take status waiter".to_string()).await;
                }
            }
            Err(err) => {
                failed().await;
                error(err.to_string()).await;
            }
        }
        finished().await;
    }
    async fn spawn(
        &self,
        command: &Command,
        environment: Option<&Environment>,
        started: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
        exit: OnceCodeCall<'async_trait>,
        stdin: InDataCall<'async_trait>,
        _stdinclose: OnceTriggerCall<'async_trait>,
        stdout: OutDataCall<'async_trait>,
        _stdoutclose: OnceTriggerCall<'async_trait>,
        stderr: OutDataCall<'async_trait>,
        _stderrclose: OnceTriggerCall<'async_trait>,
    ) {
        let pod: Api<Pod> = Api::namespaced(self.client.clone(), "melodium");

        let mut full_command = if let Some(environment) = environment {
            let mut env_command = vec!["/usr/bin/env".to_string()];

            if environment.clear_env {
                env_command.push("--ignore-environment".to_string());
            }

            if let Some(dir) = &environment.working_directory {
                env_command.push(format!("--chdir={dir}"));
            }

            for (name, val) in &environment.variables.map {
                env_command.push(format!(
                    "{name}={}",
                    if val
                        .datatype()
                        .implements(&melodium_core::common::descriptor::DataTrait::ToString)
                    {
                        melodium_core::DataTrait::to_string(val)
                    } else {
                        "".to_string()
                    }
                ));
            }

            env_command.push("--split-string".to_string());

            env_command.push(command.command.clone());

            env_command
        } else {
            vec![command.command.clone()]
        };

        full_command.extend(command.arguments.clone());

        match pod
            .exec(
                &self.pod,
                full_command,
                &AttachParams::default()
                    .container(self.container_full_name.clone())
                    .stdin(true)
                    .stdout(true)
                    .stderr(true),
            )
            .await
        {
            Ok(mut process) => {
                if let (
                    Some(status_waiter),
                    Some(mut process_stdin),
                    Some(process_stdout),
                    Some(process_stderr),
                ) = (
                    process.take_status(),
                    process.stdin(),
                    process.stdout(),
                    process.stderr(),
                ) {
                    started().await;

                    let write_stdin = async {
                        while let Ok(data) = stdin().await {
                            check!(process_stdin.write_all(&data).await);
                            check!(process_stdin.flush().await);
                        }
                    };

                    let read_stdout = async {
                        let mut process_stdout = BufReader::new(process_stdout);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = process_stdout.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(stdout(buffer[..n].iter().cloned().collect()).await);
                        }
                    };

                    let read_stderr = async {
                        let mut process_stderr = BufReader::new(process_stderr);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = process_stderr.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(stderr(buffer[..n].iter().cloned().collect()).await);
                        }
                    };

                    let status_waiter = async {
                        if let Some(status) = status_waiter.await {
                            match status.status.as_ref().map(|s| s.as_str()) {
                                Some("Success") => {
                                    completed().await;
                                    exit(status.code).await;
                                }
                                _ => {
                                    failed().await;
                                    error(
                                        status
                                            .reason
                                            .unwrap_or_else(|| "No reason provided".to_string()),
                                    )
                                    .await;
                                }
                            }
                        } else {
                            failed().await;
                            error("No output status provided".to_string()).await;
                        }
                    };

                    tokio::join!(write_stdin, read_stdout, read_stderr, status_waiter);
                } else {
                    failed().await;
                    error("Unable to take status waiter and process I/O".to_string()).await;
                }
            }
            Err(err) => {
                failed().await;
                error(err.to_string()).await;
            }
        }
        finished().await;
    }

    async fn spawn_out(
        &self,
        command: &Command,
        environment: Option<&Environment>,
        started: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
        exit: OnceCodeCall<'async_trait>,
        stdout: OutDataCall<'async_trait>,
        _stdoutclose: OnceTriggerCall<'async_trait>,
        stderr: OutDataCall<'async_trait>,
        _stderrclose: OnceTriggerCall<'async_trait>,
    ) {
        let pod: Api<Pod> = Api::namespaced(self.client.clone(), "melodium");

        let mut full_command = if let Some(environment) = environment {
            let mut env_command = vec!["/usr/bin/env".to_string()];

            if environment.clear_env {
                env_command.push("--ignore-environment".to_string());
            }

            if let Some(dir) = &environment.working_directory {
                env_command.push(format!("--chdir={dir}"));
            }

            for (name, val) in &environment.variables.map {
                env_command.push(format!(
                    "{name}={}",
                    if val
                        .datatype()
                        .implements(&melodium_core::common::descriptor::DataTrait::ToString)
                    {
                        melodium_core::DataTrait::to_string(val)
                    } else {
                        "".to_string()
                    }
                ));
            }

            env_command.push("--split-string".to_string());

            env_command.push(command.command.clone());

            env_command
        } else {
            vec![command.command.clone()]
        };

        full_command.extend(command.arguments.clone());

        match pod
            .exec(
                &self.pod,
                full_command,
                &AttachParams::default()
                    .container(self.container_full_name.clone())
                    .stdin(false)
                    .stdout(true)
                    .stderr(true),
            )
            .await
        {
            Ok(mut process) => {
                if let (Some(status_waiter), Some(process_stdout), Some(process_stderr)) =
                    (process.take_status(), process.stdout(), process.stderr())
                {
                    started().await;

                    let read_stdout = async {
                        let mut process_stdout = BufReader::new(process_stdout);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = process_stdout.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(stdout(buffer[..n].iter().cloned().collect()).await);
                        }
                    };

                    let read_stderr = async {
                        let mut process_stderr = BufReader::new(process_stderr);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = process_stderr.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(stderr(buffer[..n].iter().cloned().collect()).await);
                        }
                    };

                    let status_waiter = async {
                        if let Some(status) = status_waiter.await {
                            match status.status.as_ref().map(|s| s.as_str()) {
                                Some("Success") => {
                                    completed().await;
                                    exit(status.code).await;
                                }
                                _ => {
                                    failed().await;
                                    error(
                                        status
                                            .reason
                                            .unwrap_or_else(|| "No reason provided".to_string()),
                                    )
                                    .await;
                                }
                            }
                        } else {
                            failed().await;
                            error("No output status provided".to_string()).await;
                        }
                    };

                    tokio::join!(read_stdout, read_stderr, status_waiter);
                } else {
                    failed().await;
                    error("Unable to take status waiter and process I/O".to_string()).await;
                }
            }
            Err(err) => {
                failed().await;
                error(err.to_string()).await;
            }
        }
        finished().await;
    }
}

impl Debug for KubeExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KubeExecutor")
            .field("pod", &self.pod)
            .field("container", &self.container)
            .field("container_full_name", &self.container_full_name)
            .field("client", &"")
            .finish()
    }
}

#[derive(Debug)]
pub struct KubeFileSystem {
    #[allow(unused)]
    volume: String,
    path: PathBuf,
}

impl KubeFileSystem {
    pub async fn try_new(volume: String) -> Result<KubeFileSystem, String> {
        if Ok(true)
            != std::env::var("MELODIUM_JOB_VOLUMES")
                .map(|var| var.split(",").any(|var_volume| var_volume == volume))
        {
            return Err(format!("No volume '{volume}' listed as available"));
        }

        if let Ok(path) = std::env::var(format!("MELODIUM_JOB_VOLUME_{volume}")) {
            Ok(Self {
                volume,
                path: path.into(),
            })
        } else {
            return Err(format!("No volume '{volume}' available"));
        }
    }

    async fn full_path(&self, path: &Path) -> async_std::io::Result<PathBuf> {
        let full_path = self.path.join(path);

        if full_path.starts_with(&self.path) {
            Ok(full_path)
        } else {
            Err(async_std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        }
    }
}

#[async_trait]
impl FileSystemEngine for KubeFileSystem {
    async fn read_file(
        &self,
        path: &str,
        data: &Box<dyn Output>,
        reached: &Box<dyn Output>,
        completed: &Box<dyn Output>,
        failure: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        errors: &Box<dyn Output>,
    ) {
        let path = match self
            .full_path(&Into::<async_std::path::PathBuf>::into(path.to_string()))
            .await
        {
            Ok(path) => path,
            Err(err) => {
                let _ = failure.send_one(().into()).await;
                let _ = errors.send_one(err.to_string().into()).await;
                let _ = finished.send_one(().into()).await;
                return;
            }
        };

        let file = OpenOptions::new().read(true).open(path).await;
        match file {
            Ok(mut file) => {
                let _ = reached.send_one(().into()).await;
                reached.close().await;
                let mut vec = vec![0; 2usize.pow(20)];
                let mut fail = false;
                loop {
                    match file.read(&mut vec).await {
                        Ok(n) if n > 0 => {
                            vec.truncate(n);
                            check!(data.send_many(TransmissionValue::Byte(vec.into())).await);
                            vec = vec![0; 2usize.pow(20)];
                        }
                        Ok(_) => {
                            break;
                        }
                        Err(err) => {
                            let _ = failure.send_one(().into()).await;
                            let _ = errors.send_one(err.to_string().into()).await;
                            fail = true;
                            break;
                        }
                    }
                }
                if !fail {
                    let _ = completed.send_one(().into()).await;
                }
            }
            Err(err) => {
                let _ = failure.send_one(().into()).await;
                let _ = errors.send_one(err.to_string().into()).await;
            }
        }
        let _ = finished.send_one(().into()).await;
    }
    async fn write_file(
        &self,
        path: &str,
        append: bool,
        create: bool,
        new: bool,
        data: &Box<dyn Input>,
        amount: &Box<dyn Output>,
        completed: &Box<dyn Output>,
        failure: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        errors: &Box<dyn Output>,
    ) {
        let path = match self
            .full_path(&Into::<async_std::path::PathBuf>::into(path.to_string()))
            .await
        {
            Ok(path) => path,
            Err(err) => {
                let _ = failure.send_one(().into()).await;
                let _ = errors.send_one(err.to_string().into()).await;
                let _ = finished.send_one(().into()).await;
                return;
            }
        };

        if let Err(err) = DirBuilder::new()
            .recursive(true)
            .create(path.parent().unwrap_or(Path::new("")))
            .await
        {
            let _ = failure.send_one(().into()).await;
            let _ = errors.send_one(err.to_string().into()).await;
            let _ = finished.send_one(().into()).await;
        }

        let file = OpenOptions::new()
            .write(true)
            .append(append)
            .create(create)
            .create_new(new)
            .open(path)
            .await;
        match file {
            Ok(mut file) => {
                let mut written_amount = 0u128;
                let mut fail = false;
                while let Ok(data) = data
                    .recv_many()
                    .await
                    .map(|values| TryInto::<Vec<u8>>::try_into(values).unwrap())
                {
                    match file.write_all(&data).await {
                        Ok(_) => {
                            written_amount += data.len() as u128;
                            let _ = amount.send_one(written_amount.into()).await;
                        }
                        Err(err) => {
                            let _ = failure.send_one(().into()).await;
                            let _ = errors.send_one(err.to_string().into()).await;
                            fail = true;
                            break;
                        }
                    }
                }
                if !fail {
                    let _ = completed.send_one(().into()).await;
                }
            }
            Err(err) => {
                let _ = failure.send_one(().into()).await;
                let _ = errors.send_one(err.to_string().into()).await;
            }
        }
        let _ = finished.send_one(().into()).await;
    }
    async fn create_dir(
        &self,
        path: &str,
        recursive: bool,
        success: &Box<dyn Output>,
        failure: &Box<dyn Output>,
        error: &Box<dyn Output>,
    ) {
        let path = match self
            .full_path(&Into::<async_std::path::PathBuf>::into(path.to_string()))
            .await
        {
            Ok(path) => path,
            Err(err) => {
                let _ = failure.send_one(().into()).await;
                let _ = error.send_one(err.to_string().into()).await;
                return;
            }
        };

        match if recursive {
            fs::create_dir_all(path).await
        } else {
            fs::create_dir(path).await
        } {
            Ok(()) => {
                let _ = success.send_one(().into()).await;
            }
            Err(err) => {
                let _ = error.send_one(err.to_string().into()).await;
                let _ = failure.send_one(().into()).await;
            }
        }
    }
    async fn scan_dir(
        &self,
        path: &str,
        recursive: bool,
        follow_links: bool,
        entries: &Box<dyn Output>,
        completed: &Box<dyn Output>,
        failure: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        errors: &Box<dyn Output>,
    ) {
        let path = match self
            .full_path(&Into::<async_std::path::PathBuf>::into(path.to_string()))
            .await
        {
            Ok(path) => path,
            Err(err) => {
                let _ = failure.send_one(().into()).await;
                let _ = errors.send_one(err.to_string().into()).await;
                return;
            }
        };

        let mut dir_entries = WalkDir::new(path).filter(move |entry| async move {
            match entry.file_type().await {
                Ok(file_type) => {
                    if file_type.is_dir() {
                        if recursive {
                            Filtering::Continue
                        } else {
                            Filtering::IgnoreDir
                        }
                    } else if file_type.is_symlink() {
                        if follow_links {
                            Filtering::Continue
                        } else {
                            Filtering::IgnoreDir
                        }
                    } else {
                        Filtering::Continue
                    }
                }
                Err(_) => Filtering::Continue,
            }
        });

        let mut success = true;
        while let Some(entry) = dir_entries.next().await {
            match entry {
                Ok(entry) => check!(
                    entries
                        .send_one(entry.path().to_string_lossy().to_string().into())
                        .await
                ),
                Err(err) => {
                    success = false;
                    let _ = errors.send_one(err.to_string().into()).await;
                }
            }
        }
        let _ = finished.send_one(().into()).await;
        if success {
            let _ = completed.send_one(().into()).await;
        } else {
            let _ = failure.send_one(().into()).await;
        }
    }
}
