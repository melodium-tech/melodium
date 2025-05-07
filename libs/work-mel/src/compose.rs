use crate::api::{Access, ModeRequest, Request};
use compose_spec::{
    service::{
        ports::{Protocol, Range, ShortPort, ShortRanges},
        volumes::{
            mount::{Common, Volume},
            Mount, ShortVolume,
        },
        AbsolutePath, Cpus, Image,
    },
    Compose, Identifier, MapKey, Service, Value, Volumes,
};
use uuid::Uuid;

const IMAGES_PULL_SOURCE: &str = "quay.io/repository/melodium";

pub async fn compose(request: Request) -> Result<Access, Vec<String>> {
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

    let melodium = Service {
        container_name: Some(Identifier::new(format!("{short_id}-container")).unwrap()),
        image: Some(
            Image::parse(format!(
                "{}/melodium:{}-{}",
                IMAGES_PULL_SOURCE, request.edition, request.version
            ))
            .map_err(|err| vec![err.to_string()])?,
        ),
        command: Some(compose_spec::service::Command::List(match &request.mode {
            ModeRequest::Direct {
                entrypoint,
                project: _,
            } => vec!["run".to_string(), entrypoint.clone()],
            ModeRequest::Distribute { key } => vec![
                "dist".to_string(),
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
                "--localhost".to_string(),
            ],
        })),
        cpus: Some(Cpus::new(request.cpu as f64 / 1000f64).map_err(|err| vec![err.to_string()])?),
        mem_limit: Some(compose_spec::service::ByteValue::Megabytes(
            request.memory as u64,
        )),
        storage_opt: [(
            MapKey::new("size").unwrap(),
            Some(Value::parse(format!("{}M", request.storage))),
        )]
        .into(),
        ports: [ShortPort {
            host_ip: None,
            ranges: ShortRanges::new(
                Some(Range::new(port, None).map_err(|err| vec![err.to_string()])?),
                Range::new(8080, None).map_err(|err| vec![err.to_string()])?,
            )
            .map_err(|err| vec![err.to_string()])?,
            protocol: Some(Protocol::Tcp),
        }
        .into()]
        .into(),
        volumes: request
            .volumes
            .iter()
            .map(|volume| {
                Mount::Volume(Volume {
                    source: Some(
                        Identifier::new(format!("volume-custom-{}", volume.name)).unwrap(),
                    ),
                    common: Common {
                        target: AbsolutePath::new(format!("/media/{}", volume.name))
                            .map_err(|err| vec![err.to_string()])
                            .unwrap(),
                        read_only: false,
                        consistency: None,
                        extensions: [].into(),
                    },
                    volume: None,
                })
                .into()
            })
            .collect(),
    };

    let services = {
        request
            .containers
            .iter()
            .map(|container| (Identifier::new("common").unwrap()))
    };

    let compose_spec = Compose {
        networks: [(Identifier::new("common").unwrap(), None)].into(),
        services: services,
        volumes: volumes,
    };

    Err(vec![])
}
