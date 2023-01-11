#[derive(Debug)]
pub enum FixSerializeError{
    VersionTagNotFoundInSource,
    MessageTypeTagNotFoundInSource,
    CheckSumTagNotFoundInSource,
    InvalidCheckSum
}