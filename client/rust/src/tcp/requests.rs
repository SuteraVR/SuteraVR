use alkahest::{Formula, Serialize};
use derivative::Derivative;
use suteravr_lib::{
    clocking::{
        event_headers::EventHeader,
        oneshot_headers::{OneshotHeader, OneshotStep},
        sutera_header::SuteraHeader,
        sutera_status::SuteraStatus,
    },
    util::serialize_to_new_vec,
    SCHEMA_VERSION,
};
use tokio::sync::{mpsc, oneshot};

use super::error::TcpServerError;

pub enum Response {
    Oneshot(OneshotResponse),
    Event(EventMessage),
}

pub enum Request {
    Oneshot(OneshotRequest),
    OneshotWithReply(OneshotRequest, oneshot::Sender<Response>),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct EventMessage {
    pub sutera_header: SuteraHeader,
    pub event_header: EventHeader,
    pub payload: Vec<u8>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct OneshotResponse {
    pub sutera_header: SuteraHeader,
    pub sutera_status: SuteraStatus,
    pub oneshot_header: OneshotHeader,
    pub payload: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct OneshotRequest {
    pub sutera_header: SuteraHeader,
    pub oneshot_header: OneshotHeader,
    pub payload: Vec<u8>,
}

impl OneshotRequest {
    #[inline]
    pub fn new(
        sutera_header: SuteraHeader,
        oneshot_header: OneshotHeader,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            sutera_header,
            oneshot_header,
            payload,
        }
    }
}
impl OneshotResponse {
    #[inline]
    pub fn new(
        sutera_header: SuteraHeader,
        sutera_status: SuteraStatus,
        oneshot_header: OneshotHeader,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            sutera_header,
            sutera_status,
            oneshot_header,
            payload,
        }
    }
}

impl EventMessage {
    #[inline]
    pub fn new(sutera_header: SuteraHeader, event_header: EventHeader, payload: Vec<u8>) -> Self {
        Self {
            sutera_header,
            event_header,
            payload,
        }
    }
}

pub async fn send_oneshot_response<T: Formula + Serialize<T>>(
    response: OneshotResponse,
    reply: mpsc::Sender<Request>,
    payload: T,
) -> Result<(), TcpServerError> {
    let response = OneshotRequest {
        sutera_header: SuteraHeader {
            version: SCHEMA_VERSION,
        },
        oneshot_header: OneshotHeader {
            step: OneshotStep::Response,
            message_type: response.oneshot_header.message_type,
            message_id: response.oneshot_header.message_id,
        },
        payload: serialize_to_new_vec(payload),
    };
    reply
        .send(Request::Oneshot(response))
        .await
        .map_err(TcpServerError::CannotSendRequest)?;
    Ok(())
}
