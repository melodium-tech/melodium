
#[derive(Debug, Clone)]
pub enum TransmissionError {
    NoReceiver,
    EverythingClosed,
}

pub type SendResult = Result<(), TransmissionError>;
pub type RecvResult<T> = Result<T, TransmissionError>;
