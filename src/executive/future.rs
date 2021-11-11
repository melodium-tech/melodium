
use super::result_status::ResultStatus;

pub type ContinuousFuture = Box<dyn std::future::Future<Output = ()> + Send + Sync>;
pub type TrackFuture = Box<dyn std::future::Future<Output = ResultStatus> + Send + Sync + Unpin>;

