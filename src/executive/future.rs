
use super::result_status::ResultStatus;

pub type Future = dyn std::future::Future<Output = ResultStatus> + Send + Sync;

