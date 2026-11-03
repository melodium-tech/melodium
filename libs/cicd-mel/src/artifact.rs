//! Run artifacts of steps, pushed once their commands succeed.

use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function, mel_treatment};

/// Run artifact of a step: what is at `path` of the volume `filesystem` of the step, or of the
/// local file system of the engine running it when `filesystem` is none, pushed to
/// api.melodium.tech as the artifact `name` of the run once the commands of the step succeed
/// (see `work/artifact`).
///
/// It is a directory, its files kept at their paths in it, or a single file when `file`.
#[mel_data(
    traits (PartialEquality Serialize Deserialize Display ToString)
)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub filesystem: Option<String>,
    pub path: String,
    pub name: String,
    pub file: bool,
}

impl ToString for Artifact {
    fn to_string(&self) -> String {
        match &self.filesystem {
            Some(filesystem) => format!("{} ({filesystem}:{})", self.name, self.path),
            None => format!("{} ({})", self.name, self.path),
        }
    }
}

impl Display for Artifact {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", ToString::to_string(self))
    }
}

/// Return an artifact of the files of the directory at `path` of the volume `filesystem`,
/// under `name`.
///
/// `filesystem` none is the local file system of the engine running the step, for steps run
/// by its local executor (`executor_name` none).
#[mel_function]
pub fn artifact(filesystem: Option<string>, path: string, name: string) -> Artifact {
    Artifact {
        filesystem,
        path,
        name,
        file: false,
    }
}

/// Return an artifact of the file at `path` of the volume `filesystem`, under `name`, its
/// stable address giving the file itself.
#[mel_function]
pub fn artifact_file(filesystem: Option<string>, path: string, name: string) -> Artifact {
    Artifact {
        filesystem,
        path,
        name,
        file: true,
    }
}

/// Give the parts of an artifact.
///
/// `filesystem`, `path`, `name` and `file` are those of the artifact received through
/// `artifact`.
#[mel_treatment(
    input artifact Block<Artifact>
    output filesystem Block<Option<string>>
    output path Block<string>
    output name Block<string>
    output file Block<bool>
)]
pub async fn unpack() {
    if let Ok(artifact) = artifact.recv_one_as::<std::sync::Arc<Artifact>>().await {
        let _ = filesystem
            .send_one(Value::Option(
                artifact
                    .filesystem
                    .clone()
                    .map(|filesystem| Box::new(Value::String(filesystem))),
            ))
            .await;
        let _ = path.send_one(artifact.path.clone().into()).await;
        let _ = name.send_one(artifact.name.clone().into()).await;
        let _ = file.send_one(artifact.file.into()).await;
    }
}
