//! Scatter and gather: one track per value of a stream, as workflow languages scatter a
//! sub-workflow over the elements of a collection and gather how each ended.

use futures::{pin_mut, select, FutureExt};
use melodium_core::{
    common::descriptor::DataType,
    common::executive::{Input, Output, ResultStatus, TrackId},
    *,
};
use melodium_macro::{mel_context, mel_model, mel_treatment};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

/// Context of a scattered track.
///
/// - `scatter`: identifier of the `scatter` that made the track;
/// - `index`: position of the value of the track in the stream scattered, from 0.
#[mel_context]
pub struct Scattered {
    pub scatter: string,
    pub index: u64,
}

/// How a scattered track ended, as `gather` tells.
#[derive(Debug)]
enum Outcome {
    Completed,
    Failed(String),
    /// Ended without telling.
    Silent,
}

/// End of a scattered track, with the value it gathered, if any, and its type.
#[derive(Debug)]
struct End {
    index: u64,
    outcome: Outcome,
    value: Option<(DataType, Value)>,
}

/// Scatters values of streams over tracks of their own, and gathers how they end.
///
/// `scatter` makes a track for each value it receives, where `scattered` gives the value and the
/// value shared by all of them, and `gather` tells how the track ended. Scattered tracks get the
/// contexts of the track of `scatter`, with `@Scattered`.
///
/// Each use of `Scatter` is meant for one `scatter` (or `scatterGathering`), and the `scattered`
/// and `gather` (or `gatherValue`) of its tracks, in the same treatment, as
/// `http/server::connection` uses `incoming` and `outgoing`.
#[mel_model(
    source scattered (Scattered) (generic T () generic S () inherit) (value Block<T> shared Block<S>)
    invoke_generic_source invoke_source
)]
#[derive(Debug)]
pub struct Scatter {
    /// Types of each use of `scattered`.
    entries: Mutex<Vec<HashMap<String, DataType>>>,
    /// `scatter` waiting for their tracks to end, by identifier.
    scatters: Mutex<HashMap<String, async_channel::Sender<End>>>,
    next: AtomicU64,
}

impl Scatter {
    fn new(_model: Weak<ScatterModel>) -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            scatters: Mutex::new(HashMap::new()),
            next: AtomicU64::new(0),
        }
    }

    fn invoke_source(
        &self,
        _source: &str,
        _params: HashMap<String, Value>,
        generics: HashMap<String, DataType>,
    ) {
        self.entries.lock().unwrap().push(generics);
    }

    /// Opens a scatter, giving its identifier and where the ends of its tracks come.
    fn open(&self) -> (String, async_channel::Receiver<End>) {
        let id = self.next.fetch_add(1, Ordering::Relaxed).to_string();
        let (sender, receiver) = async_channel::unbounded();
        self.scatters.lock().unwrap().insert(id.clone(), sender);
        (id, receiver)
    }

    fn close(&self, id: &str) {
        self.scatters.lock().unwrap().remove(id);
    }

    /// Tells the scatter `id` how its track ended.
    fn report(&self, id: &str, end: End) {
        if let Some(sender) = self.scatters.lock().unwrap().get(id) {
            let _ = sender.try_send(end);
        }
    }
}

/// Scatter values over tracks of their own.
///
/// For each value received through `values`, a track is made where `scattered` gives it, with
/// the value received through `shared` (a trigger when nothing is shared), at most `parallel`
/// at once: `values` is not read further until one ends, a track ending when it tells
/// `gather` how, even if some of its treatments still finish. Tracks start in the order of `values`,
/// and end in any order, each telling how through `gather`. They get the contexts of the track
/// of `scatter`, with `@Scattered`.
///
/// Once `values` ends and every track ended, `finished` is emitted, and `completed` if they all
/// completed. If one failed, or ended without gathering, `failed` is emitted and `errors` gives
/// why. `scattered<T, S>` must be used with the same types and the same `Scatter`.
///
/// ```mermaid
/// graph LR
///     T("scatter()")
///     V["… 🟦 🟦 🟦"] -->|values| T
///     S["〈🟨〉"] -->|shared| T
///
///     T -.->|"track per 🟦"| X["scattered() … gather()"]
///     T -->|completed| C["〈🟩〉"]
///
///     style V fill:#ffffff,stroke:#ffffff
///     style S fill:#ffffff,stroke:#ffffff
///     style X fill:#ffffff,stroke:#ffffff
///     style C fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    generic T ()
    generic S ()
    model scatter Scatter
    default parallel 8
    input values Stream<T>
    input shared Block<S>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output errors Stream<string>
)]
pub async fn scatter(parallel: u32) {
    let generics = HashMap::from([("T".to_string(), T), ("S".to_string(), S)]);
    scatter_values(
        ScatterModel::into(scatter),
        track_id,
        generics,
        parallel,
        &**values,
        &**shared,
        None,
        [&**completed, &**failed, &**finished],
        &**errors,
    )
    .await
}

/// Scatter values over tracks of their own, and gather their values.
///
/// As `scatter` does, with the values the tracks give to `gatherValue` coming through
/// `gathered`, in the order the tracks end. A track gathering no value gives none, and values
/// of another type than `R` are refused, the scatter failing.
///
/// ```mermaid
/// graph LR
///     T("scatterGathering()")
///     V["… 🟦 🟦 🟦"] -->|values| T
///     S["〈🟨〉"] -->|shared| T
///
///     T -.->|"track per 🟦"| X["scattered() … gatherValue()"]
///     T -->|gathered| G["… 🟪 🟪 🟪"]
///     T -->|completed| C["〈🟩〉"]
///
///     style V fill:#ffffff,stroke:#ffffff
///     style S fill:#ffffff,stroke:#ffffff
///     style X fill:#ffffff,stroke:#ffffff
///     style G fill:#ffffff,stroke:#ffffff
///     style C fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    generic T ()
    generic S ()
    generic R ()
    model scatter Scatter
    default parallel 8
    input values Stream<T>
    input shared Block<S>
    output gathered Stream<R>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output errors Stream<string>
)]
pub async fn scatter_gathering(parallel: u32) {
    let generics = HashMap::from([("T".to_string(), T), ("S".to_string(), S)]);
    scatter_values(
        ScatterModel::into(scatter),
        track_id,
        generics,
        parallel,
        &**values,
        &**shared,
        Some((R, &**gathered)),
        [&**completed, &**failed, &**finished],
        &**errors,
    )
    .await
}

/// Makes the tracks of `scatter` and `scatterGathering`, giving their gathered values through
/// `gathered` when there is one.
async fn scatter_values(
    model: Arc<ScatterModel>,
    track_id: TrackId,
    generics: HashMap<String, DataType>,
    parallel: u32,
    values: &dyn Input,
    shared: &dyn Input,
    gathered: Option<(DataType, &dyn Output)>,
    [completed, failed, finished]: [&dyn Output; 3],
    errors: &dyn Output,
) {
    let scatter = model.inner();

    let mut problems = Vec::new();
    let mut gathered_all = true;
    if parallel == 0 {
        problems.push("parallel must be at least 1".to_string());
    } else if !scatter.entries.lock().unwrap().contains(&generics) {
        problems.push(format!(
            "no `scattered<{}, {}>` of the same `Scatter` takes the values",
            generics["T"], generics["S"]
        ));
    } else if let Ok(shared_value) = shared.recv_one().await {
        let (id, ends) = scatter.open();
        let mut running = 0;
        let mut index = 0;
        let mut values_ended = false;
        loop {
            // Ends of tracks already known are taken first.
            while let Ok(end) = ends.try_recv() {
                running -= 1;
                gathered_all &= ended(end, gathered.as_ref(), &mut problems).await;
            }
            if values_ended || running >= parallel {
                if running == 0 {
                    break;
                }
                if let Ok(end) = ends.recv().await {
                    running -= 1;
                    gathered_all &= ended(end, gathered.as_ref(), &mut problems).await;
                }
                continue;
            }
            match values.recv_one().await {
                Ok(value) => {
                    let shared_value = shared_value.clone();
                    model
                        .new_scattered(
                            Some(track_id),
                            Scattered {
                                scatter: id.clone(),
                                index,
                            },
                            &HashMap::new(),
                            &generics,
                            Some(Box::new(move |mut outputs| {
                                let value_output = outputs.get("value");
                                let shared_output = outputs.get("shared");
                                vec![Box::new(Box::pin(give(
                                    value_output,
                                    value,
                                    shared_output,
                                    shared_value,
                                )))]
                            })),
                        )
                        .await;
                    running += 1;
                    index += 1;
                }
                Err(_) => values_ended = true,
            }
        }
        scatter.close(&id);
    } else {
        problems.push("no shared value was given".to_string());
    }

    values.close();
    if let Some((_, gathered)) = &gathered {
        gathered.close().await;
    }
    if problems.is_empty() && gathered_all {
        let _ = completed.send_one(().into()).await;
    } else {
        let _ = failed.send_one(().into()).await;
        for problem in problems {
            let _ = errors.send_one(problem.into()).await;
        }
    }
    let _ = finished.send_one(().into()).await;
}

/// Takes the end of a scattered track, giving its value through `gathered` when there is one,
/// and telling whether it completed.
async fn ended(
    End {
        index,
        outcome,
        value,
    }: End,
    gathered: Option<&(DataType, &dyn Output)>,
    problems: &mut Vec<String>,
) -> bool {
    let mut gathered_well = true;
    if let (Some((expected, gathered)), Some((datatype, value))) = (gathered, value) {
        if datatype == *expected {
            let _ = gathered.send_one(value).await;
        } else {
            problems.push(format!(
                "scattered track {index} gathered a value of type {datatype}, not {expected}"
            ));
            gathered_well = false;
        }
    }
    gathered_well
        && match outcome {
            Outcome::Completed => true,
            Outcome::Failed(error) => {
                problems.push(format!("scattered track {index} failed: {error}"));
                false
            }
            Outcome::Silent => {
                problems.push(format!("scattered track {index} ended without telling how"));
                false
            }
        }
}

/// Gives the value and the shared value to a scattered track.
async fn give(
    value_output: Box<dyn Output>,
    value: Value,
    shared_output: Box<dyn Output>,
    shared: Value,
) -> ResultStatus {
    let _ = value_output.send_one(value).await;
    value_output.close().await;
    let _ = shared_output.send_one(shared).await;
    shared_output.close().await;
    ResultStatus::Ok
}

/// Tell the scatter `scatter_id` how its track `index` ended.
///
/// `gather` gives the identifier and the index from `@Scattered`. When `completed` and `failed`
/// end without any of them being emitted, the track ended without telling, and is counted as
/// failed.
#[mel_treatment(
    model scatter Scatter
    input completed Block<void>
    input failed Block<void>
    input error Block<string>
)]
pub async fn gathered(scatter_id: string, index: u64) {
    let outcome = outcome(&**completed, &**failed, &**error).await;
    ScatterModel::into(scatter).inner().report(
        &scatter_id,
        End {
            index,
            outcome,
            value: None,
        },
    );
}

/// Tell the scatter `scatter_id` how its track `index` ended, with a value.
///
/// As `gathered`, giving the value received through `value` to `scatterGathering`, whatever
/// the track's end.
#[mel_treatment(
    generic R ()
    model scatter Scatter
    input value Block<R>
    input completed Block<void>
    input failed Block<void>
    input error Block<string>
)]
pub async fn gathered_value(scatter_id: string, index: u64) {
    let (value, outcome) =
        futures::join!(value.recv_one(), outcome(&**completed, &**failed, &**error));
    ScatterModel::into(scatter).inner().report(
        &scatter_id,
        End {
            index,
            outcome,
            value: value.ok().map(|value| (R, value)),
        },
    );
}

/// How a track ended, from its `completed`, `failed` and `error`.
async fn outcome(completed: &dyn Input, failed: &dyn Input, error: &dyn Input) -> Outcome {
    let completion = completed.recv_one().fuse();
    let failure = failed.recv_one().fuse();
    pin_mut!(completion, failure);
    let has_failed = select! {
        result = completion => match result {
            Ok(_) => Some(false),
            Err(_) => failure.await.ok().map(|_| true),
        },
        result = failure => match result {
            Ok(_) => Some(true),
            Err(_) => completion.await.ok().map(|_| false),
        },
    };
    match has_failed {
        Some(false) => Outcome::Completed,
        Some(true) => Outcome::Failed(
            error
                .recv_one_as::<string>()
                .await
                .unwrap_or_else(|_| "no reason given".to_string()),
        ),
        None => Outcome::Silent,
    }
}
