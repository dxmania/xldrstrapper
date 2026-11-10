use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("timed out")]
    Timeout,
    #[error("endpoint stalled")]
    Stall,
    #[error("device disconnected")]
    Disconnected,
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("usb i/o error: {0}")]
    Io(String),
    #[error("no backend for device {0}")]
    UnsupportedDevice(crate::DeviceId),
    #[error("replay mismatch: {0}")]
    Replay(String),
}