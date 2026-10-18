pub type KResult<T> = core::result::Result<T, KError>;

#[derive(Debug)]
pub enum KError {
    Empty,
    Full,
    IndexOutOfRange,
    NotImplemented,
    NoPCIMSI,
    NoEnoughMemory,
}
