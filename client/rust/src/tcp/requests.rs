use derivative::Derivative;
use suteravr_lib::clocking::{
    event_headers::EventHeader, oneshot_headers::OneshotHeader, sutera_header::SuteraHeader,
    sutera_status::SuteraStatus,
};
use tokio::sync::oneshot;

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
