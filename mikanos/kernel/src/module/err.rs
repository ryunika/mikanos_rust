pub type KResult<T> = core::result::Result<T, KError>;

#[derive(Debug)]
pub enum KError {
    Full,
    IndexOutOfRange,
}