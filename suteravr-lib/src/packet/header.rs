use super::request_type::RequestType;
use crate::typing::player::PlayerIdentifier;

pub struct RequestHeader {
    pub sender: PlayerIdentifier,
    pub request_type: RequestType,
}
