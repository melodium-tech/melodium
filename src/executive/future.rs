
use super::result_status::ResultStatus;

pub type Future = Box<dyn std::future::Future<Output = ResultStatus> + Send + Sync + Unpin>;

