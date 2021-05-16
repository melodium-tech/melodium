
use super::result_status::ResultStatus;
use super::transmitter::Transmitter;
use super::value::Value;
use async_std::future::Future;

pub trait ModelManager {

    fn set_parameter(&self, param: &str, value: &Value);
}

pub trait TreatmentManager {
    
    fn prepare(&self) -> Vec<Box<dyn Future<Output = ResultStatus>>>;

    fn get_output(&self, output_name: &str) -> Option<Transmitter>;
    fn set_input(&self, input_name: &str) -> Result<(), ()>;
}
