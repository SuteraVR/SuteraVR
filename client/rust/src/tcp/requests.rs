use derivative::Derivative;
use suteravr_lib::{
    clocking::{
        oneshot_headers::{OneshotHeader, OneshotStep},
        sutera_header::SuteraHeader,
        sutera_status::SuteraStatus,
    },
    SCHEMA_VERSION,
};
use tokio::sync::{mpsc, oneshot};

use super::error::TcpServerError;

pub enum Request {
    Oneshot(OneshotRequest),
}

pub enum Response {
    Oneshot(OneshotResponse),
    OneshotWithReply(OneshotResponse, oneshot::Sender<Request>)
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct OneshotRequest {
    pub sutera_header: SuteraHeader,
    pub sutera_status: SuteraStatus,
    pub oneshot_header: OneshotHeader,
    pub payload: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct OneshotResponse {
    pub sutera_header: SuteraHeader,
    pub oneshot_header: OneshotHeader,
    pub payload: Vec<u8>,
}

impl OneshotRequest {
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
