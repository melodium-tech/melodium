use super::ResultStatus;

pub type ContinuousFuture = Box<dyn std::future::Future<Output = ()> + Send + Unpin>;
pub type TrackFuture = Box<dyn std::future::Future<Output = ResultStatus> + Send + Unpin>;
