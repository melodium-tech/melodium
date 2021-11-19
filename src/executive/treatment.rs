
use std::sync::Arc;
use async_std::future::Future;
use super::result_status::ResultStatus;
use super::transmitter::Transmitter;
use super::value::Value;
use super::model::Model;

pub trait Treatment{

    fn set_parameter(&self, param: &str, value: &Value);
    fn set_model(&self, name: &str, model: &Arc<dyn Model>);

    fn get_output(&self, output_name: &str) -> Option<Transmitter>;
    fn set_input(&self, input_name: &str, transmitter: Transmitter) -> Result<(), ()>;

    fn prepare(&self) -> Vec<Box<dyn Future<Output = ResultStatus>>>;
}
