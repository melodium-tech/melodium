
use std::sync::Mutex;
pub use async_std::channel::Sender;
pub use async_std::channel::Receiver;
pub use async_std::channel::{bounded, unbounded};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone)]
pub enum Transmitter {

    I8(Sender<i8>),
    I16(Sender<i16>),
    I32(Sender<i32>),
    I64(Sender<i64>),
    I128(Sender<i128>),

    U8(Sender<u8>),
    U16(Sender<u16>),
    U32(Sender<u32>),
    U64(Sender<u64>),
    U128(Sender<u128>),

    F32(Sender<f32>),
    F64(Sender<f64>),

    Bool(Sender<bool>),
    Byte(Sender<u8>),
    Char(Sender<char>),
    String(Sender<String>),

    VecI8(Sender<Vec<i8>>),
    VecI16(Sender<Vec<i16>>),
    VecI32(Sender<Vec<i32>>),
    VecI64(Sender<Vec<i64>>),
    VecI128(Sender<Vec<i128>>),

    VecU8(Sender<Vec<u8>>),
    VecU16(Sender<Vec<u16>>),
    VecU32(Sender<Vec<u32>>),
    VecU64(Sender<Vec<u64>>),
    VecU128(Sender<Vec<u128>>),

    VecF32(Sender<Vec<f32>>),
    VecF64(Sender<Vec<f64>>),

    VecBool(Sender<Vec<bool>>),
    VecByte(Sender<Vec<u8>>),
    VecChar(Sender<Vec<char>>),
    VecString(Sender<Vec<String>>),

}

const BUFFER_LIMIT: usize = 2usize.pow(20);

pub type SendResult = Result<(), TransmissionError>;
pub type RecvResult<T> = Result<T, TransmissionError>;

#[derive(Debug, Clone)]
pub enum TransmissionError {
    NoReceiver,
    EverythingClosed,
}

#[derive(Debug)]
pub struct SendTransmitter<T> {
    senders: Mutex<Vec<Sender<Vec<T>>>>,
    buffer: Mutex<Vec<T>>,

    has_receivers: AtomicBool,
}

impl<T: Clone> SendTransmitter<T> {

    pub fn new() -> Self {
        Self {
            senders: Mutex::new(Vec::new()),
            buffer: Mutex::new(Vec::with_capacity(BUFFER_LIMIT)),
            has_receivers: AtomicBool::new(false),
        }
    }

    pub fn add_transmitter(&self, transmitter: &RecvTransmitter<T>) {

        let sender = transmitter.get_sender();
        self.senders.lock().unwrap().push(sender);

        self.has_receivers.store(true, Ordering::Relaxed);
    }

    pub async fn send(&self, data: T) -> SendResult {

        if !self.has_receivers.load(Ordering::Relaxed) {
            return Err(TransmissionError::NoReceiver)
        }
        else
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push(data);
        }
        
        self.check_send().await
    }

    pub async fn send_multiple(&self, data: Vec<T>) -> SendResult {

        if !self.has_receivers.load(Ordering::Relaxed) {
            return Err(TransmissionError::NoReceiver)
        }
        else
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.extend(data);
        }
        
        self.check_send().await
    }

    async fn check_send(&self) -> SendResult {

        let buffer_len = self.buffer.lock().unwrap().len();

        if buffer_len >= BUFFER_LIMIT {

            self.do_send().await
        }
        else {
            Ok(())
        }
    }

    async fn do_send(&self) -> SendResult {

        let buffer = self.buffer.lock().unwrap().clone();

        let mut statuses = Vec::new();
        let senders = self.senders.lock().unwrap().clone();
        for sender in senders.iter() {
            statuses.push(
                match sender.send(buffer.clone()).await {
                    Ok(()) => true,
                    Err(_) => false,
                }
            );
        };

        let status = if let Some(_) = statuses.iter().find(|s| **s) {
            Ok(())
        }
        else {
            Err(TransmissionError::EverythingClosed)
        };

        self.buffer.lock().unwrap().clear();

        return status;
    }

    pub async fn close(&self) {

        // In closing we don't care for send result
        let _result = self.do_send().await;

        self.senders.lock().unwrap().iter().for_each(|s| { s.close(); } );
    }
}

trait SenderGetter<T> {
    fn get_sender(&self) -> Sender<Vec<T>>;
}

#[derive(Debug)]
pub struct RecvTransmitter<T> {
    receiver: Receiver<Vec<T>>,
    sender: Sender<Vec<T>>,
}

impl<T: Clone> RecvTransmitter<T> {

    pub fn new() -> Self {
        let (sender, receiver) = unbounded();

        Self {
            sender,
            receiver,
        }
    }

    pub async fn receive_multiple(&self) -> RecvResult<Vec<T>> {
        
        match self.receiver.recv().await {
            Ok(v) => Ok(v),
            Err(_) => Err(TransmissionError::EverythingClosed),
        }
    }

    pub fn close(&self) {
        self.receiver.close();
    }
}

impl<T> SenderGetter<T> for RecvTransmitter<T> {

    fn get_sender(&self) -> Sender<Vec<T>> {
        self.sender.clone()
    }
}
