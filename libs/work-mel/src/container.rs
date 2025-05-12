use crate::compose::Executor;
use async_std::{
    fs::{self, DirBuilder, OpenOptions},
    io::BufReader,
    path::{Path, PathBuf},
};
use async_trait::async_trait;
use async_walkdir::{Filtering, WalkDir};
use fs_mel::filesystem::{self, FileSystemEngine};
use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use melodium_macro::check;
use process_mel::{
    command::Command,
    environment::{environment_variable_regex, Environment},
    exec::*,
};
use regex::{Captures, Replacer};
use std::{
    collections::{HashMap, HashSet},
    process::Stdio,
};

#[derive(Debug)]
pub struct ContainerExecutor {
    executor: Executor,
    #[allow(unused)]
    name: String,
    container_name: String,
}

impl ContainerExecutor {
    pub async fn try_new(container: String) -> Result<ContainerExecutor, String> {
        let executor = match std::env::var("MELODIUM_JOB_EXECUTOR").as_deref() {
            Ok("podman") => Executor::Podman,
            Ok("docker") => Executor::Docker,
            Ok(other) => {
                return Err(format!(
                    "Executor '{other}' not known as manageable executor"
                ))
            }
            Err(_) => return Err("Executor name not set".to_string()),
        };

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
            Ok(Self {
                executor,
                name: container,
                container_name: container_full_name,
            })
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
                dst.push_str(
                    self.variables
                        .get(&caps[1].to_string())
                        .map(|str| str.clone())
                        .unwrap_or_default()
                        .as_str(),
                );
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
        let mut map = HashMap::new();

        for key in keys {
            let var_command = vec![
                "exec".to_string(),
                self.container_name.clone(),
                "echo".to_string(),
                format!("${key}"),
            ];

            match async_std::process::Command::new(self.executor.to_string())
                .args(var_command)
                .output()
                .await
            {
                Ok(output) if output.status.success() => {
                    let variable = String::from_utf8_lossy(&output.stdout).to_string();
                    map.insert(key, variable);
                }
                _ => {}
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
impl ExecutorEngine for ContainerExecutor {
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
        let mut arguments = vec!["exec".to_string()];
        if let Some(environment) = environment {
            if let Some(dir) = &environment.working_directory {
                arguments.push(format!("--workdir={dir}"));
            }

            if environment.clear_env {
                // Nothing doable
            }

            if environment.expand_variables {
                let variables = self
                    .manage_variable_substitution(&environment.variables.map)
                    .await;
                for (name, val) in variables {
                    arguments.push("--env".to_string());
                    arguments.push(format!("{name}={val}"));
                }
            } else {
                for (name, val) in &environment.variables.map {
                    arguments.push("--env".to_string());
                    arguments.push(format!("{name}={val}"));
                }
            }
        }

        arguments.push(self.container_name.clone());

        arguments.push(command.command.clone());

        arguments.extend(command.arguments.clone());

        started().await;
        match async_std::process::Command::new(self.executor.to_string())
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
        {
            Ok(output) => match output.status.success() {
                true => {
                    completed().await;
                    exit(output.status.code()).await;
                }
                false => {
                    failed().await;
                    error(String::from_utf8_lossy(&output.stderr).to_string()).await;
                }
            },
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
        stdinclose: OnceTriggerCall<'async_trait>,
        stdout: OutDataCall<'async_trait>,
        stdoutclose: OnceTriggerCall<'async_trait>,
        stderr: OutDataCall<'async_trait>,
        stderrclose: OnceTriggerCall<'async_trait>,
    ) {
        let mut arguments = vec!["exec".to_string(), "--interactive".to_string()];
        if let Some(environment) = environment {
            if let Some(dir) = &environment.working_directory {
                arguments.push(format!("--workdir={dir}"));
            }

            if environment.clear_env {
                // Nothing doable
            }

            if environment.expand_variables {
                let variables = self
                    .manage_variable_substitution(&environment.variables.map)
                    .await;
                for (name, val) in variables {
                    arguments.push("--env".to_string());
                    arguments.push(format!("{name}={val}"));
                }
            } else {
                for (name, val) in &environment.variables.map {
                    arguments.push("--env".to_string());
                    arguments.push(format!("{name}={val}"));
                }
            }
        }

        arguments.push(self.container_name.clone());

        arguments.push(command.command.clone());

        arguments.extend(command.arguments.clone());

        match async_std::process::Command::new(self.executor.to_string())
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                started().await;

                let child_stdin = child.stdin.take();
                let child_stdout = child.stdout.take();
                let child_stderr = child.stderr.take();

                let write_stdin = async {
                    if let Some(mut child_stdin) = child_stdin {
                        while let Ok(data) = stdin().await {
                            check!(child_stdin.write_all(&data).await);
                            check!(child_stdin.flush().await);
                        }

                        let _ = child_stdin.close().await;
                    } else {
                        stdinclose().await;
                    }
                };

                let read_stdout = async {
                    if let Some(child_stdout) = child_stdout {
                        let mut child_stdout = BufReader::new(child_stdout);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = child_stdout.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(stdout(buffer[..n].iter().cloned().collect()).await);
                        }
                    } else {
                        stdoutclose().await;
                    }
                };

                let read_stderr = async {
                    if let Some(child_stderr) = child_stderr {
                        let mut child_stderr = BufReader::new(child_stderr);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = child_stderr.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(stderr(buffer[..n].iter().cloned().collect()).await);
                        }
                    } else {
                        stderrclose().await;
                    }
                };

                let status = async {
                    match child.status().await {
                        Ok(status) => {
                            completed().await;
                            exit(status.code()).await;
                        }
                        Err(err) => {
                            failed().await;
                            error(err.to_string()).await;
                        }
                    }
                };

                let _ = futures::join!(status, write_stdin, read_stdout, read_stderr);
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
        stdoutclose: OnceTriggerCall<'async_trait>,
        stderr: OutDataCall<'async_trait>,
        stderrclose: OnceTriggerCall<'async_trait>,
    ) {
        let mut arguments = vec!["exec".to_string()];
        if let Some(environment) = environment {
            if let Some(dir) = &environment.working_directory {
                arguments.push(format!("--workdir={dir}"));
            }

            if environment.clear_env {
                // Nothing doable
            }

            if environment.expand_variables {
                let variables = self
                    .manage_variable_substitution(&environment.variables.map)
                    .await;
                for (name, val) in variables {
                    arguments.push("--env".to_string());
                    arguments.push(format!("{name}={val}"));
                }
            } else {
                for (name, val) in &environment.variables.map {
                    arguments.push("--env".to_string());
                    arguments.push(format!("{name}={val}"));
                }
            }
        }

        arguments.push(self.container_name.clone());

        arguments.push(command.command.clone());

        arguments.extend(command.arguments.clone());

        match async_std::process::Command::new(self.executor.to_string())
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                started().await;

                let child_stdout = child.stdout.take();
                let child_stderr = child.stderr.take();

                let read_stdout = async {
                    if let Some(child_stdout) = child_stdout {
                        let mut child_stdout = BufReader::new(child_stdout);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = child_stdout.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(stdout(buffer[..n].iter().cloned().collect()).await);
                        }
                    } else {
                        stdoutclose().await;
                    }
                };

                let read_stderr = async {
                    if let Some(child_stderr) = child_stderr {
                        let mut child_stderr = BufReader::new(child_stderr);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = child_stderr.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(stderr(buffer[..n].iter().cloned().collect()).await);
                        }
                    } else {
                        stderrclose().await;
                    }
                };

                let status = async {
                    match child.status().await {
                        Ok(status) => {
                            completed().await;
                            exit(status.code()).await;
                        }
                        Err(err) => {
                            failed().await;
                            error(err.to_string()).await;
                        }
                    }
                };

                let _ = futures::join!(status, read_stdout, read_stderr);
            }
            Err(err) => {
                failed().await;
                error(err.to_string()).await;
            }
        }
        finished().await;
    }
}

#[derive(Debug)]
pub struct ContainerFileSystem {
    #[allow(unused)]
    volume: String,
    path: PathBuf,
}

impl ContainerFileSystem {
    pub async fn try_new(volume: String) -> Result<ContainerFileSystem, String> {
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
impl FileSystemEngine for ContainerFileSystem {
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
