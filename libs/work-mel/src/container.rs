use crate::compose::Executor;
use async_trait::async_trait;
#[cfg(feature = "real")]
use futures::io::BufReader;
use futures::{AsyncReadExt, AsyncWriteExt};
use melodium_macro::check;
use process_mel::{
    command::Command,
    environment::{environment_variable_regex, Environment, RevealedSecrets},
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
    executor_entrypoint: String,
    #[allow(unused)]
    name: String,
    container_name: String,
}

#[cfg(feature = "real")]
impl ContainerExecutor {
    pub async fn try_new(container: String) -> Result<ContainerExecutor, String> {
        let executor = match std::env::var("MELODIUM_RUN_EXECUTOR").as_deref() {
            Ok("podman") => Executor::Podman,
            Ok("docker") => Executor::Docker,
            Ok(other) => {
                return Err(format!(
                    "Executor '{other}' not known as manageable executor"
                ))
            }
            Err(_) => return Err("Executor name not set".to_string()),
        };

        let executor_entrypoint =
            if let Ok(entrypoint) = std::env::var("MELODIUM_RUN_EXECUTOR_ENTRYPOINT") {
                entrypoint
            } else {
                executor.to_string()
            };

        if Ok(true)
            != std::env::var("MELODIUM_RUN_CONTAINERS").map(|var| {
                var.split(",")
                    .any(|var_container| var_container == container)
            })
        {
            return Err(format!("No container '{container}' listed as available"));
        }

        if let Ok(container_full_name) =
            std::env::var(format!("MELODIUM_RUN_CONTAINER_{container}"))
        {
            Ok(Self {
                executor,
                executor_entrypoint,
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
            let mut var_command = vec![];
            if self.executor == Executor::Podman {
                var_command.push("--remote".to_string());
            }
            var_command.push("exec".to_string());
            var_command.push(self.container_name.clone());
            var_command.push("echo".to_string());
            var_command.push(format!("${key}"));

            match async_process::Command::new(self.executor_entrypoint.clone())
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
        secrets: &RevealedSecrets,
        terminate: OnceRecvCall<'async_trait>,
        started: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
        exit: OnceCodeCall<'async_trait>,
    ) {
        #[cfg(feature = "real")]
        {
            let mut arguments = vec![];
            if self.executor == Executor::Podman {
                arguments.push("--remote".to_string());
            }
            arguments.push("exec".to_string());

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

            // The client takes their values from its own environment, out of its arguments.
            for (name, _) in &secrets.variables {
                arguments.push("--env".to_string());
                arguments.push(name.clone());
            }
            if secrets.stdin.is_some() && !arguments.iter().any(|arg| arg == "--interactive") {
                arguments.push("--interactive".to_string());
            }

            arguments.push(self.container_name.clone());

            arguments.push(command.command.clone());

            arguments.extend(command.arguments.clone());

            match async_process::Command::new(self.executor_entrypoint.clone())
                .args(arguments)
                .envs(
                    secrets
                        .variables
                        .iter()
                        .map(|(name, value)| (name, value.as_str())),
                )
                .stdin(if secrets.stdin.is_some() {
                    Stdio::piped()
                } else {
                    Stdio::null()
                })
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(mut child) => {
                    use futures::{pin_mut, select, FutureExt};

                    started().await;
                    write_secret_stdin(child.stdin.take(), secrets).await;

                    let mut to_terminate = false;

                    {
                        let status = async { child.status().await }.fuse();
                        let terminate = async {
                            to_terminate = terminate().await;
                            to_terminate
                        }
                        .fuse();

                        pin_mut!(status, terminate);

                        loop {
                            select! {
                                to_terminate = terminate => if to_terminate { break },
                                status = status => {
                                    match status {
                                        Ok(status) => {
                                            completed().await;
                                            exit(status.code()).await;
                                        }
                                        Err(err) => {
                                            failed().await;
                                            error(err.to_string()).await;
                                        }
                                    }
                                    break
                                },
                                complete => break,
                            }
                        }
                    }
                    if to_terminate {
                        let _ = child.kill();
                    }
                }
                Err(err) => {
                    failed().await;
                    error(err.to_string()).await;
                }
            }
            finished().await;
        }
        #[cfg(feature = "mock")]
        {
            let _ = failed().await;
            let _ = error("Mock mode".to_string()).await;
            let _ = finished().await;
        }
    }

    async fn spawn(
        &self,
        command: &Command,
        environment: Option<&Environment>,
        secrets: &RevealedSecrets,
        terminate: OnceRecvCall<'async_trait>,
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
        #[cfg(feature = "real")]
        {
            let mut arguments = vec![];
            if self.executor == Executor::Podman {
                arguments.push("--remote".to_string());
            }
            arguments.push("exec".to_string());
            arguments.push("--interactive".to_string());

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

            // The client takes their values from its own environment, out of its arguments.
            for (name, _) in &secrets.variables {
                arguments.push("--env".to_string());
                arguments.push(name.clone());
            }
            if secrets.stdin.is_some() && !arguments.iter().any(|arg| arg == "--interactive") {
                arguments.push("--interactive".to_string());
            }

            arguments.push(self.container_name.clone());

            arguments.push(command.command.clone());

            arguments.extend(command.arguments.clone());

            match async_process::Command::new(self.executor_entrypoint.clone())
                .args(arguments)
                .envs(
                    secrets
                        .variables
                        .iter()
                        .map(|(name, value)| (name, value.as_str())),
                )
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    started().await;

                    let mut to_terminate = false;

                    {
                        use futures::{pin_mut, select, FutureExt};

                        let child_stdin = child.stdin.take();
                        let child_stdout = child.stdout.take();
                        let child_stderr = child.stderr.take();

                        let write_stdin = async {
                            if let Some(mut child_stdin) = child_stdin {
                                let input_written = match &secrets.stdin {
                                    Some(input) => {
                                        child_stdin.write_all(input.as_bytes()).await.is_ok()
                                            && child_stdin.flush().await.is_ok()
                                    }
                                    None => true,
                                };
                                if input_written {
                                    while let Ok(data) = stdin().await {
                                        check!(child_stdin.write_all(&data).await);
                                        check!(child_stdin.flush().await);
                                    }
                                }

                                let _ = child_stdin.close().await;
                            } else {
                                stdinclose().await;
                            }
                        }
                        .fuse();

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
                        }
                        .fuse();

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
                        }
                        .fuse();

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
                        }
                        .fuse();

                        let terminate = async {
                            to_terminate = terminate().await;
                            to_terminate
                        }
                        .fuse();

                        pin_mut!(write_stdin, read_stdout, read_stderr, status, terminate);

                        loop {
                            select! {
                                () = write_stdin => {},
                                () = read_stdout => {},
                                () = read_stderr => {},
                                () = status => break,
                                to_terminate = terminate => if to_terminate { break },
                                complete => break,
                            }
                        }
                    }
                    if to_terminate {
                        let _ = child.kill();
                    }
                }
                Err(err) => {
                    failed().await;
                    error(err.to_string()).await;
                }
            }
            finished().await;
        }
        #[cfg(feature = "mock")]
        {
            let _ = failed().await;
            let _ = error("Mock mode".to_string()).await;
            let _ = finished().await;
        }
    }

    async fn spawn_out(
        &self,
        command: &Command,
        environment: Option<&Environment>,
        secrets: &RevealedSecrets,
        terminate: OnceRecvCall<'async_trait>,
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
        #[cfg(feature = "real")]
        {
            let mut arguments = vec![];
            if self.executor == Executor::Podman {
                arguments.push("--remote".to_string());
            }
            arguments.push("exec".to_string());

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

            // The client takes their values from its own environment, out of its arguments.
            for (name, _) in &secrets.variables {
                arguments.push("--env".to_string());
                arguments.push(name.clone());
            }
            if secrets.stdin.is_some() && !arguments.iter().any(|arg| arg == "--interactive") {
                arguments.push("--interactive".to_string());
            }

            arguments.push(self.container_name.clone());

            arguments.push(command.command.clone());

            arguments.extend(command.arguments.clone());

            match async_process::Command::new(self.executor_entrypoint.clone())
                .args(arguments)
                .envs(
                    secrets
                        .variables
                        .iter()
                        .map(|(name, value)| (name, value.as_str())),
                )
                .stdin(if secrets.stdin.is_some() {
                    Stdio::piped()
                } else {
                    Stdio::null()
                })
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    started().await;
                    write_secret_stdin(child.stdin.take(), secrets).await;

                    let mut to_terminate = false;

                    {
                        use futures::{pin_mut, select, FutureExt};

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
                        }
                        .fuse();

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
                        }
                        .fuse();

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
                        }
                        .fuse();

                        let terminate = async {
                            to_terminate = terminate().await;
                            to_terminate
                        }
                        .fuse();

                        pin_mut!(read_stdout, read_stderr, status, terminate);

                        loop {
                            select! {
                                () = read_stdout => {},
                                () = read_stderr => {},
                                () = status => break,
                                to_terminate = terminate => if to_terminate { break },
                                complete => break,
                            }
                        }
                    }
                    if to_terminate {
                        let _ = child.kill();
                    }
                }
                Err(err) => {
                    failed().await;
                    error(err.to_string()).await;
                }
            }
            finished().await;
        }
        #[cfg(feature = "mock")]
        {
            let _ = failed().await;
            let _ = error("Mock mode".to_string()).await;
            let _ = finished().await;
        }
    }
}

/// Writes the secret input, if any, to `stdin`, then closes it.
#[cfg(feature = "real")]
async fn write_secret_stdin(stdin: Option<async_process::ChildStdin>, secrets: &RevealedSecrets) {
    if let (Some(mut stdin), Some(input)) = (stdin, &secrets.stdin) {
        let _ = stdin.write_all(input.as_bytes()).await;
        let _ = stdin.close().await;
    }
}
