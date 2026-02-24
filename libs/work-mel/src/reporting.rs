use crate::api::{DistributionResponse, LocalEnd, LocalLaunched, ModeRequest, Request};
use async_std::channel::Receiver;
use melodium_core::common::{descriptor::Version, executive::Log};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportingRequest {
    pub run_id: Uuid,
    pub group_id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reporting {
    pub run_id: Uuid,
    pub group_id: Uuid,
    pub dashboard: Option<String>,
    pub logs: Option<PushSpecs>,
    pub debug: Option<PushSpecs>,
    pub program: Option<PushSpecs>,
}

pub struct StatusReporting {
    pub launched: Option<
        Box<
            dyn FnOnce(
                Result<(), String>,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
        >,
    >,
    pub ended: Option<
        Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>>,
    >,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum PushSpecs {
    PresignedPutS3 {
        uri: String,
        headers: HashMap<String, String>,
    },
    PresignedPostS3 {
        uri: String,
        fields: HashMap<String, String>,
        path: String,
    },
}

static CLIENT: std::sync::LazyLock<Result<reqwest::Client, String>> =
    std::sync::LazyLock::new(|| {
        reqwest::Client::builder()
            .user_agent(crate::USER_AGENT)
            .build()
            .map_err(|err| err.to_string())
    });

pub async fn request_reporting(
    enable_reports: bool,
    enable_status: bool,
    request: ReportingRequest,
    version: &Version,
    mode: ModeRequest,
) -> Result<(Reporting, StatusReporting), String> {
    if enable_status {
        match generic_async_http_client::Request::post(&format!(
            "{api_url}/execution/run/start",
            api_url = crate::API_URL.as_str()
        ))
        .add_header("User-Agent", crate::USER_AGENT)
        .map_err(|err| err.to_string())?
        .add_header(
            "Authorization",
            format!(
                "Bearer {api_token}",
                api_token = crate::API_TOKEN
                    .as_ref()
                    .map(|token| token.as_str())
                    .unwrap_or(&"")
            )
            .as_bytes(),
        )
        .map_err(|err| err.to_string())?
        .add_header("Content-Type", "application/json")
        .map_err(|err| err.to_string())?
        .body(
            serde_json::to_string(&Request {
                config: None,
                id: Some(request.run_id),
                organization_id: None,
                edition: None,
                version: version.to_string(),
                mode,
                max_duration: None,
                memory: None,
                cpu: None,
                storage: None,
                arch: None,
                volumes: vec![],
                containers: vec![],
                service_containers: vec![],
                tags: vec![],
                group_id: Some(request.group_id),
                parent_id: None,
                local_exec: true,
            })
            .unwrap(),
        )
        .map_err(|err| err.to_string())?
        .exec()
        .await
        {
            Ok(mut response) => {
                if response.status_code() == 200 || response.status_code() == 409 {
                    // Nothing
                } else {
                    return match response.text().await {
                        Ok(body) => Err(format!(
                            "Server {} response: {body}",
                            response.status_code()
                        )),
                        Err(error) => Err(error.to_string()),
                    };
                }
            }
            Err(error) => return Err(error.to_string()),
        }
    }

    let mut reporting = Reporting {
        run_id: request.run_id,
        group_id: request.group_id,
        dashboard: None,
        logs: None,
        debug: None,
        program: None,
    };
    if enable_reports {
        match generic_async_http_client::Request::post(&format!(
            "{api_url}/execution/report/request",
            api_url = crate::API_URL.as_str()
        ))
        .add_header("User-Agent", crate::USER_AGENT)
        .map_err(|err| err.to_string())?
        .add_header(
            "Authorization",
            format!(
                "Bearer {api_token}",
                api_token = crate::API_TOKEN
                    .as_ref()
                    .map(|token| token.as_str())
                    .unwrap_or(&"")
            )
            .as_bytes(),
        )
        .map_err(|err| err.to_string())?
        .add_header("Content-Type", "application/json")
        .map_err(|err| err.to_string())?
        .body(serde_json::to_string(&request).unwrap())
        .map_err(|err| err.to_string())?
        .exec()
        .await
        {
            Ok(mut response) => {
                if response.status_code() == 200 {
                    match response.json::<Reporting>().await {
                        Ok(resp_reporting) => {
                            reporting = resp_reporting;
                        }
                        Err(error) => return Err(error.to_string()),
                    }
                } else {
                    match response.text().await {
                        Ok(body) => {
                            return Err(format!(
                                "Server {} response: {body}",
                                response.status_code()
                            ))
                        }
                        Err(error) => return Err(error.to_string()),
                    }
                }
            }
            Err(error) => return Err(error.to_string()),
        }
    }

    let mut status_reporting = StatusReporting {
        launched: None,
        ended: None,
    };
    if enable_status {
        let run_id = request.run_id.clone();
        status_reporting.launched = Some(Box::new(move |result| {
            Box::pin(async move {
                let call = async || {
                    generic_async_http_client::Request::post(&format!(
                        "{api_url}/execution/run/launched",
                        api_url = crate::API_URL.as_str()
                    ))
                    .add_header("User-Agent", crate::USER_AGENT)?
                    .add_header(
                        "Authorization",
                        format!(
                            "Bearer {api_token}",
                            api_token = crate::API_TOKEN
                                .as_ref()
                                .map(|token| token.as_str())
                                .unwrap_or(&"")
                        )
                        .as_bytes(),
                    )?
                    .add_header("Content-Type", "application/json")?
                    .body(
                        serde_json::to_string(&match result {
                            Ok(_) => LocalLaunched {
                                run_id,
                                response: DistributionResponse::Started(None),
                            },
                            Err(err) => LocalLaunched {
                                run_id,
                                response: DistributionResponse::Error(vec![err]),
                            },
                        })
                        .unwrap(),
                    )?
                    .exec()
                    .await
                };
                let _ = call().await;
            })
        }));
        status_reporting.ended = Some(Box::new(move || {
            Box::pin(async move {
                let call = async || {
                    generic_async_http_client::Request::post(&format!(
                        "{api_url}/execution/run/ended",
                        api_url = crate::API_URL.as_str()
                    ))
                    .add_header("User-Agent", crate::USER_AGENT)?
                    .add_header(
                        "Authorization",
                        format!(
                            "Bearer {api_token}",
                            api_token = crate::API_TOKEN
                                .as_ref()
                                .map(|token| token.as_str())
                                .unwrap_or(&"")
                        )
                        .as_bytes(),
                    )?
                    .add_header("Content-Type", "application/json")?
                    .body(
                        serde_json::to_string(&LocalEnd {
                            run_id,
                            result: crate::api::DistributionResult::Success(None),
                        })
                        .unwrap(),
                    )?
                    .exec()
                    .await
                };
                let _ = call().await;
            })
        }));
    }

    Ok((reporting, status_reporting))
}

pub async fn report_logs(specs: PushSpecs, logs: Receiver<Log>) {
    match specs {
        PushSpecs::PresignedPostS3 { uri, fields, path } => {
            let mut buffer_logs = Vec::with_capacity(1000);
            let mut timestamp = std::time::SystemTime::now();
            let mut batch_index: u128 = 0;
            while let Ok(log) = logs.recv().await {
                buffer_logs.push(log);
                if buffer_logs.len() >= 1000
                    || timestamp.elapsed().unwrap_or_default().as_secs() >= 5
                {
                    let mut fields = fields.clone();
                    fields.insert("key".to_string(), format!("{path}/logs_{batch_index}.json"));

                    if let Err(err) = send_logs_to_s3(&uri, fields, &buffer_logs).await {
                        eprintln!("Failed to send logs to S3: {err}");
                    }

                    buffer_logs.clear();
                    timestamp = std::time::SystemTime::now();
                    batch_index += 1;
                }
            }
            // Send any remaining logs
            if !buffer_logs.is_empty() {
                let mut fields = fields.clone();
                fields.insert("key".to_string(), format!("{path}/logs_{batch_index}.json"));

                if let Err(err) = send_logs_to_s3(&uri, fields, &buffer_logs).await {
                    eprintln!("Failed to send logs to S3: {err}");
                }
            }
        }
        _ => {}
    }
}

pub async fn report_debug(specs: PushSpecs, events: Receiver<melodium_engine::debug::Event>) {
    match specs {
        PushSpecs::PresignedPostS3 { uri, fields, path } => {
            let mut buffer_events = Vec::with_capacity(1000);
            let mut timestamp = std::time::SystemTime::now();
            let mut batch_index: u128 = 0;
            while let Ok(event) = events.recv().await {
                buffer_events.push(melodium_share::Event::from(&event));
                if buffer_events.len() >= 1000
                    || timestamp.elapsed().unwrap_or_default().as_secs() >= 5
                {
                    let mut fields = fields.clone();
                    fields.insert(
                        "key".to_string(),
                        format!("{path}/debug_{batch_index}.json"),
                    );

                    if let Err(err) = send_debug_to_s3(&uri, fields, &buffer_events).await {
                        eprintln!("Failed to send debug events to S3: {err}");
                    }

                    buffer_events.clear();
                    timestamp = std::time::SystemTime::now();
                    batch_index += 1;
                }
            }
            // Send any remaining logs
            if !buffer_events.is_empty() {
                let mut fields = fields.clone();
                fields.insert(
                    "key".to_string(),
                    format!("{path}/debug_{batch_index}.json"),
                );

                if let Err(err) = send_debug_to_s3(&uri, fields, &buffer_events).await {
                    eprintln!("Failed to send debug events to S3: {err}");
                }
            }
        }
        _ => {}
    }
}

pub async fn report_program(specs: PushSpecs, program: melodium_share::ProgramDump) {
    match specs {
        PushSpecs::PresignedPutS3 { uri, headers } => {
            let client = match CLIENT.as_ref() {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("Failed to create HTTP client: {err}");
                    return;
                }
            };

            let request = client.put(&uri);
            let request = headers
                .into_iter()
                .fold(request, |req, (key, value)| req.header(key, value));
            match request
                .body(serde_json::to_vec(&program).unwrap())
                .send()
                .await
            {
                Ok(response) => {
                    if !response.status().is_success() {
                        eprintln!(
                            "Failed to upload program dump: Server responded with status code {}",
                            response.status()
                        );
                    }
                }
                Err(err) => eprintln!("Failed to upload program dump: {err}"),
            }
        }
        _ => {}
    }
}

async fn send_logs_to_s3(
    uri: &str,
    fields: HashMap<String, String>,
    logs: &Vec<Log>,
) -> Result<(), String> {
    let mut form = reqwest::multipart::Form::new();
    for (key, value) in fields {
        form = form.text(key, value);
    }

    form = form.part(
        "file",
        reqwest::multipart::Part::bytes(serde_json::to_vec(logs).unwrap())
            .file_name("logs.json")
            .mime_str("application/json")
            .unwrap(),
    );

    CLIENT
        .as_ref()?
        .post(uri)
        .multipart(form)
        .send()
        .await
        .map(|_| ())
        .map_err(|err| err.to_string())
}

async fn send_debug_to_s3(
    uri: &str,
    fields: HashMap<String, String>,
    events: &Vec<melodium_share::Event>,
) -> Result<(), String> {
    let mut form = reqwest::multipart::Form::new();
    for (key, value) in fields {
        form = form.text(key, value);
    }

    form = form.part(
        "file",
        reqwest::multipart::Part::bytes(serde_json::to_vec(events).unwrap())
            .file_name("debug.json")
            .mime_str("application/json")
            .unwrap(),
    );

    CLIENT
        .as_ref()?
        .post(uri)
        .multipart(form)
        .send()
        .await
        .map(|_| ())
        .map_err(|err| err.to_string())
}
