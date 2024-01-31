use derivative::Derivative;
use suteravr_lib::{
    clocking::{
        oneshot_headers::{OneshotHeader, OneshotStep},
        sutera_header::SuteraHeader,
        sutera_status::SuteraStatus,
    },
    SCHEMA_VERSION,
};
use tokio::sync::mpsc;

use super::error::TcpServerError;

pub enum Request {
    Oneshot(OneshotRequest),
}

pub enum Response {
    Oneshot(OneshotResponse),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct OneshotRequest {
    pub sutera_header: SuteraHeader,
    pub sutera_status: SuteraStatus,
    pub oneshot_header: OneshotHeader,
    pub payload: Vec<u8>,

    #[derivative(Debug = "ignore")]
    reply: mpsc::Sender<Response>,
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
        reply: mpsc::Sender<Response>,
    ) -> Self {
        Self {
            sutera_header,
            sutera_status,
            oneshot_header,
            payload,
            reply,
        }
    }

    #[inline]
    pub async fn send_reply(self, payload: Vec<u8>) -> Result<(), TcpServerError> {
        let response = self.to_reply(payload);
        self.reply
            .send(Response::Oneshot(response))
            .await
            .map_err(TcpServerError::CannotSendResponse)?;
        Ok(())
    }

    #[inline]
    pub fn to_reply(&self, payload: Vec<u8>) -> OneshotResponse {
        OneshotResponse {
            sutera_header: SuteraHeader {
                version: SCHEMA_VERSION,
            },
            oneshot_header: OneshotHeader {
                step: OneshotStep::Response,
                message_type: self.oneshot_header.message_type,
                message_id: self.oneshot_header.message_id,
            },
            payload,
        }
    }
}
