use async_std::{
    fs::{self, DirBuilder, OpenOptions},
    io::{ReadExt, WriteExt},
    path::{Path, PathBuf},
    stream::StreamExt,
};
use async_trait::async_trait;
use async_walkdir::{Filtering, WalkDir};
use core::fmt::Debug;
use fs_mel::filesystem::{self, FileSystemEngine};
use k8s_openapi::api::core::v1::Pod;
use kube::{api::AttachParams, Api, Client};
use melodium_macro::check;
use process_mel::{
    command::Command,
    environment::{environment_variable_regex, Environment},
    exec::*,
};
use regex::{Captures, Replacer};
use std::collections::{HashMap, HashSet};
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

    async fn manage_variable_substitution(
        &self,
        variables: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let names = Self::get_used_variables(variables);

        let environment_variables = self.get_environment_variables(names).await;

        struct VarReplacer<'a> {
            variables: &'a HashMap<String, String>,
        }

        impl Replacer for VarReplacer<'_> {
            fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
                *dst = self
                    .variables
                    .get(&caps[1].to_string())
                    .map(|str| str.clone())
                    .unwrap_or_default();
            }
        }

        let regex = environment_variable_regex();
        let mut substitued_variables = HashMap::new();
        for (name, content) in variables.iter() {
            let expanded = regex
                .replace_all(
                    content,
                    VarReplacer {
                        variables: &environment_variables,
                    },
                )
                .to_string();
            substitued_variables.insert(name.clone(), expanded);
        }

        substitued_variables
    }

    async fn get_environment_variables(&self, keys: Vec<String>) -> HashMap<String, String> {
        let pod: Api<Pod> = Api::namespaced(self.client.clone(), "melodium");

        let mut map = HashMap::new();

        for key in keys {
            let var_command = vec![
                "/usr/bin/env".to_string(),
                "--split-string".to_string(),
                "sh".to_string(),
                "-c".to_string(),
                format!("echo ${key}"),
            ];

            match pod
                .exec(
                    &self.pod,
                    var_command,
                    &AttachParams::default()
                        .stdin(false)
                        .stdout(true)
                        .stderr(false)
                        .container(self.container_full_name.clone()),
                )
                .await
            {
                Ok(mut process) => {
                    if let Some(process_stdout) = process.stdout() {
                        let read_stdout = async {
                            let mut process_stdout = BufReader::new(process_stdout);

                            let mut content = String::new();
                            match process_stdout.read_to_string(&mut content).await {
                                Ok(s) => {
                                    content.pop();
                                    eprintln!("key {key}: {s} bytes read, content: {content}");
                                    content
                                }
                                Err(e) => {
                                    eprintln!("key {key}: error '{e}'");
                                    String::new()
                                }
                            }
                        };

                        let variable = read_stdout.await;
                        map.insert(key, variable);
                    }
                }
                Err(_err) => {}
            }
        }

        map
    }

    fn get_used_variables(map: &HashMap<String, String>) -> Vec<String> {
        let mut set = HashSet::new();

        for (_, var) in map {
            let regex = environment_variable_regex();
            for capture in regex.captures_iter(var) {
                if let Some(name) = capture.get(1) {
                    let name = name.as_str().to_string();
                    set.insert(name);
                }
            }
        }

        set.into_iter().collect()
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

            if let Some(dir) = &environment.working_directory {
                env_command.push(format!("--chdir={dir}"));
            }

            env_command.push("--split-string".to_string());

            if environment.clear_env {
                env_command.push("--ignore-environment".to_string());
            }

            if environment.expand_variables {
                let variables = self
                    .manage_variable_substitution(&environment.variables.map)
                    .await;
                for (name, val) in variables {
                    env_command.push(format!("{name}={val}"));
                }
            } else {
                for (name, val) in &environment.variables.map {
                    env_command.push(format!("{name}={val}"));
                }
            }

            env_command.push(command.command.clone());

            env_command
        } else {
            vec![command.command.clone()]
        };

        full_command.extend(command.arguments.clone());

        eprintln!("{full_command:?}");

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
                                if let Some(causes) = status.details.map(|d| d.causes).flatten() {
                                    let code = causes
                                        .iter()
                                        .filter(|cause| cause.reason.as_deref() == Some("ExitCode"))
                                        .map(|cause| {
                                            cause.message.as_ref().map(|msg| msg.parse().ok())
                                        })
                                        .next()
                                        .flatten()
                                        .flatten();
                                    exit(code).await;
                                } else {
                                    exit(None).await;
                                }
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

            if let Some(dir) = &environment.working_directory {
                env_command.push(format!("--chdir={dir}"));
            }

            env_command.push("--split-string".to_string());

            if environment.clear_env {
                env_command.push("--ignore-environment".to_string());
            }

            if environment.expand_variables {
                let variables = self
                    .manage_variable_substitution(&environment.variables.map)
                    .await;
                for (name, val) in variables {
                    env_command.push(format!("{name}={val}"));
                }
            } else {
                for (name, val) in &environment.variables.map {
                    env_command.push(format!("{name}={val}"));
                }
            }

            env_command.push(command.command.clone());

            env_command
        } else {
            vec![command.command.clone()]
        };

        full_command.extend(command.arguments.clone());

        eprintln!("{full_command:?}");

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
                                    if let Some(causes) = status.details.map(|d| d.causes).flatten()
                                    {
                                        let code = causes
                                            .iter()
                                            .filter(|cause| {
                                                cause.reason.as_deref() == Some("ExitCode")
                                            })
                                            .map(|cause| {
                                                cause.message.as_ref().map(|msg| msg.parse().ok())
                                            })
                                            .next()
                                            .flatten()
                                            .flatten();
                                        exit(code).await;
                                    } else {
                                        exit(None).await;
                                    }
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

            if let Some(dir) = &environment.working_directory {
                env_command.push(format!("--chdir={dir}"));
            }

            env_command.push("--split-string".to_string());

            if environment.clear_env {
                env_command.push("--ignore-environment".to_string());
            }

            if environment.expand_variables {
                let variables = self
                    .manage_variable_substitution(&environment.variables.map)
                    .await;
                for (name, val) in variables {
                    env_command.push(format!("{name}={val}"));
                }
            } else {
                for (name, val) in &environment.variables.map {
                    env_command.push(format!("{name}={val}"));
                }
            }

            env_command.push(command.command.clone());

            env_command
        } else {
            vec![command.command.clone()]
        };

        full_command.extend(command.arguments.clone());

        eprintln!("{full_command:?}");

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
                                    if let Some(causes) = status.details.map(|d| d.causes).flatten()
                                    {
                                        let code = causes
                                            .iter()
                                            .filter(|cause| {
                                                cause.reason.as_deref() == Some("ExitCode")
                                            })
                                            .map(|cause| {
                                                cause.message.as_ref().map(|msg| msg.parse().ok())
                                            })
                                            .next()
                                            .flatten()
                                            .flatten();
                                        exit(code).await;
                                    } else {
                                        exit(None).await;
                                    }
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
        data: filesystem::OutDataCall<'async_trait>,
        reached: filesystem::OnceTriggerCall<'async_trait>,
        reachedclose: filesystem::OnceTriggerCall<'async_trait>,
        completed: filesystem::OnceTriggerCall<'async_trait>,
        failed: filesystem::OnceTriggerCall<'async_trait>,
        finished: filesystem::OnceTriggerCall<'async_trait>,
        errors: filesystem::OutMessageCall<'async_trait>,
    ) {
        let path = match self
            .full_path(&Into::<async_std::path::PathBuf>::into(path.to_string()))
            .await
        {
            Ok(path) => path,
            Err(err) => {
                failed().await;
                let _ = errors(err.to_string()).await;
                finished().await;
                return;
            }
        };

        let file = OpenOptions::new().read(true).open(path).await;
        match file {
            Ok(mut file) => {
                reached().await;
                reachedclose().await;
                let mut vec = vec![0; 2usize.pow(20)];
                let mut fail = false;
                loop {
                    match file.read(&mut vec).await {
                        Ok(n) if n > 0 => {
                            vec.truncate(n);
                            check!(data(vec.into()).await);
                            vec = vec![0; 2usize.pow(20)];
                        }
                        Ok(_) => {
                            break;
                        }
                        Err(err) => {
                            failed().await;
                            let _ = errors(err.to_string()).await;
                            fail = true;
                            break;
                        }
                    }
                }
                if !fail {
                    completed().await;
                }
            }
            Err(err) => {
                failed().await;
                let _ = errors(err.to_string()).await;
            }
        }
        finished().await;
    }
    async fn write_file(
        &self,
        path: &str,
        append: bool,
        create: bool,
        new: bool,
        data: filesystem::InDataCall<'async_trait>,
        amount: filesystem::OutU128Call<'async_trait>,
        completed: filesystem::OnceTriggerCall<'async_trait>,
        failed: filesystem::OnceTriggerCall<'async_trait>,
        finished: filesystem::OnceTriggerCall<'async_trait>,
        errors: filesystem::OutMessageCall<'async_trait>,
    ) {
        let path = match self
            .full_path(&Into::<async_std::path::PathBuf>::into(path.to_string()))
            .await
        {
            Ok(path) => path,
            Err(err) => {
                failed().await;
                let _ = errors(err.to_string()).await;
                finished().await;
                return;
            }
        };

        if let Err(err) = DirBuilder::new()
            .recursive(true)
            .create(path.parent().unwrap_or(Path::new("")))
            .await
        {
            failed().await;
            let _ = errors(err.to_string()).await;
            finished().await;
        } else {
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
                    while let Ok(data) = data().await {
                        match file.write_all(&data).await {
                            Ok(_) => {
                                written_amount += data.len() as u128;
                                let _ = amount(written_amount).await;
                            }
                            Err(err) => {
                                failed().await;
                                let _ = errors(err.to_string()).await;
                                fail = true;
                                break;
                            }
                        }
                    }
                    if !fail {
                        completed().await;
                    }
                }
                Err(err) => {
                    failed().await;
                    let _ = errors(err.to_string()).await;
                }
            }
            finished().await;
        }
    }
    async fn create_dir(
        &self,
        path: &str,
        recursive: bool,
        success: filesystem::OnceTriggerCall<'async_trait>,
        failed: filesystem::OnceTriggerCall<'async_trait>,
        error: filesystem::OnceMessageCall<'async_trait>,
    ) {
        let path = match self
            .full_path(&Into::<async_std::path::PathBuf>::into(path.to_string()))
            .await
        {
            Ok(path) => path,
            Err(err) => {
                failed().await;
                error(err.to_string()).await;
                return;
            }
        };

        match if recursive {
            fs::create_dir_all(path).await
        } else {
            fs::create_dir(path).await
        } {
            Ok(()) => {
                success().await;
            }
            Err(err) => {
                error(err.to_string()).await;
                failed().await;
            }
        }
    }
    async fn scan_dir(
        &self,
        path: &str,
        recursive: bool,
        follow_links: bool,
        entries: filesystem::OutMessageCall<'async_trait>,
        completed: filesystem::OnceTriggerCall<'async_trait>,
        failed: filesystem::OnceTriggerCall<'async_trait>,
        finished: filesystem::OnceTriggerCall<'async_trait>,
        errors: filesystem::OutMessageCall<'async_trait>,
    ) {
        let path = match self
            .full_path(&Into::<async_std::path::PathBuf>::into(path.to_string()))
            .await
        {
            Ok(path) => path,
            Err(err) => {
                failed().await;
                let _ = errors(err.to_string()).await;
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
                Ok(entry) => check!(entries(entry.path().to_string_lossy().to_string()).await),
                Err(err) => {
                    success = false;
                    let _ = errors(err.to_string()).await;
                }
            }
        }
        finished().await;
        if success {
            completed().await;
        } else {
            failed().await;
        }
    }
}
