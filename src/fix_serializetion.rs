use crate::FixSerializeError;

pub trait FixDeserializeModel {
    fn deserialize_fix(&self) -> Vec<u8>;
}

pub trait FixSerializeModel {
    fn serialize_fix<T>(payload: &[u8]) -> Result<T, FixSerializeError>
    where
        T: FixSerializeModel;
}
