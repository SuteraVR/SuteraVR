use suteravr_lib::schema_oneshot::OneshotVariants;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializeError {
    #[error("Bad Payload")]
    BadPayload,
}

pub struct SerializedPayload<'a> {
    pub variant: OneshotVariants,
    pub payload: &'a u8,
}

fn serialize(payload: &[u8]) -> Result<SerializedPayload, SerializeError> {
    unimplemented!()
}
