//! Mélodium main library and binary
//!
//! Please refer to the [Mélodium Project](https://melodium.tech/),
//! [Mélodium Book](https://doc.melodium.tech/book/en/),
//! or [Mélodium Documentation](https://doc.melodium.tech/latest/en/) for usage.
//!
//! Please refer to the [crates.io page](https://crates.io/crates/melodium) or
//! [project repository](https://gitlab.com/melodium/melodium) for compilation or development information.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use async_std::{
    channel::{unbounded, Receiver},
    fs::OpenOptions,
    io::{BufWriter, WriteExt},
};
use colored::Colorize;
use futures::StreamExt;
use melodium_common::{
    descriptor::{
        Collection, Identifier, LoadingError, LoadingResult, Package, PackageRequirement,
        VersionReq,
    },
    executive::{Level, Log, Value},
};
use melodium_engine::{
    debug::{DebugLevel, Event},
    LogicResult,
};
pub use melodium_loader::LoadingConfig;
use melodium_loader::{Compo, Loader, PackageInfo};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "fs")]
pub mod new;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const TARGET: &'static str = env!("TARGET");
pub const TARGET_FEATURES: &'static str = env!("TARGET_FEATURE");
pub const BUILD_HOST: &'static str = env!("HOST");

pub fn load_all(
    mut config: LoadingConfig,
) -> LoadingResult<(Vec<Arc<dyn PackageInfo>>, Arc<Collection>)> {
    config.extend(core_config());

    let loader = Loader::new(config);
    loader.load_all().and_then(|_| {
        loader
            .build()
            .and_then(|coll| LoadingResult::new_success((loader.packages(), coll)))
    })
}

/*

This former function is kept as implementation idea for direct load and launch of directly available elements.

pub fn load_entry(
    mut config: LoadingConfig,
    identifier: &Identifier,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    config.extend(core_config());


    let loader = Loader::new(config);
    loader
        .load_package(&PackageRequirement {
            package: identifier.root().to_string(),
            version_requirement: VersionReq::parse(">=0.0.0").unwrap(),
        })
        .and_then(|_| loader.load(identifier))
        .and_then(|_| loader.build())
}*/

pub fn load_compo(
    content: &str,
    entrypoint: &str,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    config.extend(core_config());

    Compo::parse(content).and_then(|compo| {
        let loader = Loader::new(config);
        loader
            .load_package(&PackageRequirement {
                package: compo.name,
                version_requirement: VersionReq::parse(&format!("={}", compo.version)).unwrap(),
            })
            .and_then(|pkg| {
                if let Some(main) = pkg.entrypoints().get(entrypoint) {
                    loader
                        .load(&main.into())
                        .and(LoadingResult::new_success(pkg))
                } else {
                    LoadingResult::new_failure(LoadingError::no_entry_point_provided(245))
                }
            })
            .and_then(|pkg| {
                loader
                    .build()
                    .and_then(|collection| LoadingResult::new_success((pkg, collection)))
            })
    })
}

pub fn load_compo_all_entrypoints(
    content: &str,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    config.extend(core_config());

    Compo::parse(content).and_then(|compo| {
        let loader = Loader::new(config);
        loader
            .load_package(&PackageRequirement {
                package: compo.name,
                version_requirement: VersionReq::parse(&format!("={}", compo.version)).unwrap(),
            })
            .and_then(|pkg| {
                let mut result = LoadingResult::new_success(Arc::clone(&pkg));
                for (_, id) in pkg.entrypoints() {
                    result = result.and(
                        loader
                            .load(&id.into())
                            .and(LoadingResult::new_success(Arc::clone(&pkg))),
                    )
                }
                result
            })
            .and_then(|pkg| {
                loader
                    .build()
                    .and_then(|collection| LoadingResult::new_success((pkg, collection)))
            })
    })
}

pub fn load_compo_force_entrypoint(
    content: &str,
    identifier: &Identifier,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    config.extend(core_config());

    Compo::parse(content).and_then(|compo| {
        let loader = Loader::new(config);
        loader
            .load_package(&PackageRequirement {
                package: compo.name,
                version_requirement: VersionReq::parse(&format!("={}", compo.version)).unwrap(),
            })
            .and_then(|pkg| {
                loader
                    .load(&identifier.into())
                    .and(LoadingResult::new_success(pkg))
            })
            .and_then(|pkg| {
                loader
                    .build()
                    .and_then(|collection| LoadingResult::new_success((pkg, collection)))
            })
    })
}

pub fn load_raw(
    raw: Arc<Vec<u8>>,
    entrypoint: &str,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    config.extend(core_config());

    let loader = Loader::new(config);
    loader
        .load_raw(raw)
        .and_then(|pkg| {
            if let Some(main) = pkg.entrypoints().get(entrypoint) {
                loader
                    .load(&main.into())
                    .and(LoadingResult::new_success(pkg))
            } else {
                LoadingResult::new_failure(LoadingError::no_entry_point_provided(238))
            }
        })
        .and_then(|pkg| {
            loader
                .build()
                .and_then(|collection| LoadingResult::new_success((pkg, collection)))
        })
}

pub fn load_raw_all_entrypoints(
    raw: Arc<Vec<u8>>,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    config.extend(core_config());

    let loader = Loader::new(config);
    loader
        .load_raw(raw)
        .and_then(|pkg| {
            let mut result = LoadingResult::new_success(Arc::clone(&pkg));
            for (_, id) in pkg.entrypoints() {
                result = result.and(
                    loader
                        .load(&id.into())
                        .and(LoadingResult::new_success(Arc::clone(&pkg))),
                )
            }
            result
        })
        .and_then(|pkg| {
            loader
                .build()
                .and_then(|collection| LoadingResult::new_success((pkg, collection)))
        })
}

pub fn load_raw_force_entrypoint(
    raw: Arc<Vec<u8>>,
    identifier: &Identifier,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    config.extend(core_config());

    let loader = Loader::new(config);
    loader
        .load_raw(raw)
        .and_then(|pkg| {
            loader
                .load(&identifier.into())
                .and(LoadingResult::new_success(pkg))
        })
        .and_then(|pkg| {
            loader
                .build()
                .and_then(|collection| LoadingResult::new_success((pkg, collection)))
        })
}

pub fn load_file(
    file: PathBuf,
    entrypoint: &str,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    if file
        .file_name()
        .map(|file_name| file_name == "Compo.toml")
        .unwrap_or(false)
    {
        config.search_locations.push(file.clone());
        match std::fs::read_to_string(&file) {
            Ok(content) => load_compo(&content, entrypoint, config),
            Err(err) => LoadingResult::new_failure(LoadingError::unreachable_file(
                246,
                file,
                err.to_string(),
            )),
        }
    } else {
        match std::fs::read(&file) {
            Ok(content) => load_raw(Arc::new(content), entrypoint, config),
            Err(err) => LoadingResult::new_failure(LoadingError::unreachable_file(
                193,
                file,
                err.to_string(),
            )),
        }
    }
}

pub fn load_file_all_entrypoints(
    file: PathBuf,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    if file
        .file_name()
        .map(|file_name| file_name == "Compo.toml")
        .unwrap_or(false)
    {
        config.search_locations.push(file.clone());
        match std::fs::read_to_string(&file) {
            Ok(content) => load_compo_all_entrypoints(&content, config),
            Err(err) => LoadingResult::new_failure(LoadingError::unreachable_file(
                247,
                file,
                err.to_string(),
            )),
        }
    } else {
        match std::fs::read(&file) {
            Ok(content) => load_raw_all_entrypoints(Arc::new(content), config),
            Err(err) => LoadingResult::new_failure(LoadingError::unreachable_file(
                244,
                file,
                err.to_string(),
            )),
        }
    }
}

pub fn load_file_force_entrypoint(
    file: PathBuf,
    identifier: &Identifier,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    if file
        .file_name()
        .map(|file_name| file_name == "Compo.toml")
        .unwrap_or(false)
    {
        config.search_locations.push(file.clone());
        match std::fs::read_to_string(&file) {
            Ok(content) => load_compo_force_entrypoint(&content, identifier, config),
            Err(err) => LoadingResult::new_failure(LoadingError::unreachable_file(
                248,
                file,
                err.to_string(),
            )),
        }
    } else {
        match std::fs::read(&file) {
            Ok(content) => load_raw_force_entrypoint(Arc::new(content), identifier, config),
            Err(err) => LoadingResult::new_failure(LoadingError::unreachable_file(
                243,
                file,
                err.to_string(),
            )),
        }
    }
}

pub async fn launch(
    collection: Arc<Collection>,
    identifier: &Identifier,
    parameters: HashMap<String, Value>,
    log_path: Option<PathBuf>,
    debug_path: Option<PathBuf>,
    api_report: bool,
) -> LogicResult<()> {
    let engine = melodium_engine::new_engine(collection, Level::Trace, DebugLevel::Detailed);

    let mut monitoring = futures::stream::FuturesUnordered::new();

    let (logs_stdout_sender, logs_stdout_receiver) = unbounded();
    engine.add_logs_listener(logs_stdout_sender);
    monitoring.push(async_std::task::spawn(async move {
        display_logs(logs_stdout_receiver).await
    }));
    if let Some(log_path) = log_path.clone() {
        let (logs_write_sender, logs_write_receiver) = unbounded();
        engine.add_logs_listener(logs_write_sender);
        monitoring.push(async_std::task::spawn(async move {
            write_logs(log_path, logs_write_receiver).await
        }));
    }
    if let Some(debug_path) = debug_path {
        let (debug_write_sender, debug_write_receiver) = unbounded();
        engine.add_debug_listener(debug_write_sender);
        monitoring.push(async_std::task::spawn(async move {
            write_debug(debug_path, debug_write_receiver).await
        }));
    }

    #[cfg(feature = "work-mel")]
    if api_report {
        let reporting_request = work_mel::reporting::ReportingRequest {
            run_id: *melodium_engine::execution_run_id(),
            group_id: *melodium_engine::execution_group_id(),
        };
        let reporting = work_mel::reporting::request_reporting(reporting_request).await;
        match reporting {
            Ok(reporting) => {
                if let Some(logs_specs) = reporting.logs {
                    let (logs_report_sender, logs_report_receiver) = unbounded();
                    engine.add_logs_listener(logs_report_sender);
                    monitoring.push(async_std::task::spawn(async move {
                        work_mel::reporting::report_logs(logs_specs, logs_report_receiver).await
                    }));
                }
                if let Some(debug_specs) = reporting.debug {
                    let (debug_report_sender, debug_report_receiver) = unbounded();
                    engine.add_debug_listener(debug_report_sender);
                    monitoring.push(async_std::task::spawn(async move {
                        work_mel::reporting::report_debug(debug_specs, debug_report_receiver).await
                    }));
                }
                if let Some(program_specs) = reporting.program {
                    let program_dump = work_mel::reporting::ProgramDump {
                        collection: melodium_share::Collection::from_entrypoint(
                            &engine.collection(),
                            identifier,
                        ),
                        entrypoint: melodium_share::Identifier::from(identifier),
                        parameters: parameters
                            .iter()
                            .map(|(k, v)| (k.clone(), melodium_share::RawValue::from(v)))
                            .collect(),
                    };
                    monitoring.push(async_std::task::spawn(async move {
                        work_mel::reporting::report_program(program_specs, program_dump).await
                    }));
                }
            }
            Err(error) => {
                eprintln!("Failed to request reporting: {error}");
            }
        }
    }

    let result = engine.genesis(&identifier, parameters);
    if result.is_failure() {
        return result;
    } else {
        engine.live().await;
        engine.end().await;
    }

    while let Some(_) = monitoring.next().await {}

    LogicResult::new_success(())
}

pub fn core_config() -> LoadingConfig {
    LoadingConfig {
        core_packages: core_packages(),
        search_locations: Vec::new(),
        raw_elements: Vec::new(),
    }
}

pub fn core_packages() -> Vec<Arc<dyn Package>> {
    #[allow(unused_mut)]
    let mut packages = Vec::new();
    packages.push(std_mel::__mel_package::package());

    #[cfg(feature = "cicd-mel")]
    packages.push(cicd_mel::__mel_package::package());
    #[cfg(feature = "distrib-mel")]
    packages.push(distrib_mel::__mel_package::package());
    #[cfg(feature = "encoding-mel")]
    packages.push(encoding_mel::__mel_package::package());
    #[cfg(feature = "fs-mel")]
    packages.push(fs_mel::__mel_package::package());
    #[cfg(feature = "http-mel")]
    packages.push(http_mel::__mel_package::package());
    #[cfg(feature = "javascript-mel")]
    packages.push(javascript_mel::__mel_package::package());
    #[cfg(feature = "json-mel")]
    packages.push(json_mel::__mel_package::package());
    #[cfg(feature = "net-mel")]
    packages.push(net_mel::__mel_package::package());
    #[cfg(feature = "process-mel")]
    packages.push(process_mel::__mel_package::package());
    #[cfg(feature = "regex-mel")]
    packages.push(regex_mel::__mel_package::package());
    #[cfg(feature = "sql-mel")]
    packages.push(sql_mel::__mel_package::package());
    #[cfg(feature = "work-mel")]
    packages.push(work_mel::__mel_package::package());

    packages
}

pub async fn display_logs(receiver: Receiver<Log>) {
    while let Ok(log) = receiver.recv().await {
        display_log(&log);
    }
}

fn display_log(log: &Log) {
    match log.level {
        Level::Error => {
            println!(
                "[{}] {}: {}: {}",
                log.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "error".bold().red(),
                log.label,
                log.message
            )
        }
        Level::Warning => {
            println!(
                "[{}] {}: {}: {}",
                log.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "warning".bold().yellow(),
                log.label,
                log.message
            )
        }
        Level::Info => {
            println!(
                "[{}] {}: {}: {}",
                log.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "info".bold().blue(),
                log.label,
                log.message
            )
        }
        Level::Debug => {
            println!(
                "[{}] {}: {}: {}",
                log.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "debug".bold().purple(),
                log.label,
                log.message
            )
        }
        Level::Trace => {
            println!(
                "[{}] {}: {}: {}",
                log.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "trace".bold().dimmed(),
                log.label,
                log.message
            )
        }
    }
}

pub async fn write_logs(path: PathBuf, receiver: Receiver<Log>) {
    if let Some(parent) = path.parent() {
        let _ = async_std::fs::create_dir_all(parent).await;
    }

    if let Ok(log_file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
    {
        let mut log_file = BufWriter::new(log_file);

        while let Ok(log) = receiver.recv().await {
            let line = format!(
                "[{}] {}: {}: {}",
                log.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                log.level,
                log.label,
                log.message,
            );
            let _ = log_file.write(line.as_bytes()).await;
        }

        let _ = log_file.flush().await;
    }
}

pub async fn write_debug(path: PathBuf, receiver: Receiver<Event>) {
    if let Some(parent) = path.parent() {
        let _ = async_std::fs::create_dir_all(parent).await;
    }

    if let Ok(debug_file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
    {
        let mut debug_file = BufWriter::new(debug_file);
        let mut first = true;
        let _ = debug_file.write("[".as_bytes()).await;
        while let Ok(debug) = receiver.recv().await {
            if !first {
                let _ = debug_file.write(",".as_bytes()).await;
            } else {
                first = false;
            }
            let event = melodium_share::Event::from(&debug);
            let line = serde_json::to_string(&event)
                .unwrap_or_else(|_| "\"<failed to serialize debug event>\"".to_string());
            let _ = debug_file.write(line.as_bytes()).await;
        }
        let _ = debug_file.write("]".as_bytes()).await;

        let _ = debug_file.flush().await;
    }
}
