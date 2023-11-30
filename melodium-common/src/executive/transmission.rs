#[derive(Debug, Clone)]
pub enum TransmissionError {
    NoReceiver,
    EverythingClosed,
    NoData,
}

pub type SendResult = Result<(), TransmissionError>;
pub type RecvResult<T> = Result<T, TransmissionError>;
