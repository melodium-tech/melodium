
use super::result_status::ResultStatus;
use super::transmitter::Transmitter;
use async_std::future::Future;

pub trait Manager {
    
    fn prepare(&self) -> Vec<Box<dyn Future<Output = ResultStatus>>>;

    fn get_output(&self, output_name: &str) -> Option<Transmitter>;
}
