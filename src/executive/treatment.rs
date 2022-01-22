
use std::fmt::Debug;
use crate::logic::descriptor::CoreTreatmentDescriptor;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_std::future::Future;
use super::result_status::ResultStatus;
use super::transmitter::*;
use super::future::TrackFuture;
use super::value::Value;
use super::model::Model;

pub trait Treatment{

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor>;

    fn set_parameter(&self, param: &str, value: &Value);
    fn set_model(&self, name: &str, model: &Arc<dyn Model>);

    fn set_output(&self, output_name: &str, transmitter: Vec<Transmitter>);
    fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>>;

    fn prepare(&self) -> Vec<TrackFuture>;
}

pub trait TreatmentImpl : Debug {
    fn process(&self);
}

#[derive(Debug)]
pub enum Input {
    I8(Arc<RecvTransmitter<i8>>),
    I16(Arc<RecvTransmitter<i16>>),
    I32(Arc<RecvTransmitter<i32>>),
    I64(Arc<RecvTransmitter<i64>>),
    I128(Arc<RecvTransmitter<i128>>),

    U8(Arc<RecvTransmitter<u8>>),
    U16(Arc<RecvTransmitter<u16>>),
    U32(Arc<RecvTransmitter<u32>>),
    U64(Arc<RecvTransmitter<u64>>),
    U128(Arc<RecvTransmitter<u128>>),

    F32(Arc<RecvTransmitter<f32>>),
    F64(Arc<RecvTransmitter<f64>>),

    Bool(Arc<RecvTransmitter<bool>>),
    Byte(Arc<RecvTransmitter<u8>>),
    Char(Arc<RecvTransmitter<char>>),
    String(Arc<RecvTransmitter<String>>),

    VecI8(Arc<RecvTransmitter<Vec<i8>>>),
    VecI16(Arc<RecvTransmitter<Vec<i16>>>),
    VecI32(Arc<RecvTransmitter<Vec<i32>>>),
    VecI64(Arc<RecvTransmitter<Vec<i64>>>),
    VecI128(Arc<RecvTransmitter<Vec<i128>>>),

    VecU8(Arc<RecvTransmitter<Vec<u8>>>),
    VecU16(Arc<RecvTransmitter<Vec<u16>>>),
    VecU32(Arc<RecvTransmitter<Vec<u32>>>),
    VecU64(Arc<RecvTransmitter<Vec<u64>>>),
    VecU128(Arc<RecvTransmitter<Vec<u128>>>),

    VecF32(Arc<RecvTransmitter<Vec<f32>>>),
    VecF64(Arc<RecvTransmitter<Vec<f64>>>),

    VecBool(Arc<RecvTransmitter<Vec<bool>>>),
    VecByte(Arc<RecvTransmitter<Vec<u8>>>),
    VecChar(Arc<RecvTransmitter<Vec<char>>>),
    VecString(Arc<RecvTransmitter<Vec<String>>>),
}

#[derive(Debug)]
pub enum Output {
    I8(Arc<SendTransmitter<i8>>),
    I16(Arc<SendTransmitter<i16>>),
    I32(Arc<SendTransmitter<i32>>),
    I64(Arc<SendTransmitter<i64>>),
    I128(Arc<SendTransmitter<i128>>),

    U8(Arc<SendTransmitter<u8>>),
    U16(Arc<SendTransmitter<u16>>),
    U32(Arc<SendTransmitter<u32>>),
    U64(Arc<SendTransmitter<u64>>),
    U128(Arc<SendTransmitter<u128>>),

    F32(Arc<SendTransmitter<f32>>),
    F64(Arc<SendTransmitter<f64>>),

    Bool(Arc<SendTransmitter<bool>>),
    Byte(Arc<SendTransmitter<u8>>),
    Char(Arc<SendTransmitter<char>>),
    String(Arc<SendTransmitter<String>>),

    VecI8(Arc<SendTransmitter<Vec<i8>>>),
    VecI16(Arc<SendTransmitter<Vec<i16>>>),
    VecI32(Arc<SendTransmitter<Vec<i32>>>),
    VecI64(Arc<SendTransmitter<Vec<i64>>>),
    VecI128(Arc<SendTransmitter<Vec<i128>>>),

    VecU8(Arc<SendTransmitter<Vec<u8>>>),
    VecU16(Arc<SendTransmitter<Vec<u16>>>),
    VecU32(Arc<SendTransmitter<Vec<u32>>>),
    VecU64(Arc<SendTransmitter<Vec<u64>>>),
    VecU128(Arc<SendTransmitter<Vec<u128>>>),

    VecF32(Arc<SendTransmitter<Vec<f32>>>),
    VecF64(Arc<SendTransmitter<Vec<f64>>>),

    VecBool(Arc<SendTransmitter<Vec<bool>>>),
    VecByte(Arc<SendTransmitter<Vec<u8>>>),
    VecChar(Arc<SendTransmitter<Vec<char>>>),
    VecString(Arc<SendTransmitter<Vec<String>>>),
}

#[derive(Debug)]
pub struct TreatmentHost {

    descriptor: Arc<CoreTreatmentDescriptor>,

    models: Mutex<HashMap<String, Arc<dyn Model>>>,
    parameters: Mutex<HashMap<String, Value>>,

    inputs: Mutex<HashMap<String, Input>>,
    outputs: Mutex<HashMap<String, Output>>,
}

impl TreatmentHost {

    pub fn new(descriptor: Arc<CoreTreatmentDescriptor>) -> Self {
        todo!()
    }
}
