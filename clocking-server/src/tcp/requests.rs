use alkahest::{Formula, Serialize};
use derivative::Derivative;
use suteravr_lib::{
    clocking::{
        event_headers::{EventRequest, EventResponse},
        oneshot_headers::{OneshotHeader, OneshotStep},
        sutera_header::SuteraHeader,
        sutera_status::{SuteraStatus, SuteraStatusError},
    },
    util::serialize_to_new_vec,
    SCHEMA_VERSION,
};
use tokio::sync::mpsc;

use crate::errors::TcpServerError;

pub enum Request {
    Oneshot(OneshotRequest),
    Event(EventRequest),
}

pub enum Response {
    Oneshot(OneshotResponse),
    Event(EventResponse),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct OneshotRequest {
    pub sutera_header: SuteraHeader,
    pub oneshot_header: OneshotHeader,
    pub payload: Vec<u8>,

    #[derivative(Debug = "ignore")]
    reply: mpsc::Sender<Response>,
}

#[derive(Debug, PartialEq)]
pub struct OneshotResponse {
    pub sutera_header: SuteraHeader,
    pub sutera_status: SuteraStatus,
    pub oneshot_header: OneshotHeader,
    pub payload: Vec<u8>,
}

impl OneshotRequest {
    #[inline]
    pub fn new(
        sutera_header: SuteraHeader,
        oneshot_header: OneshotHeader,
        payload: Vec<u8>,
        reply: mpsc::Sender<Response>,
    ) -> Self {
        Self {
            sutera_header,
            oneshot_header,
            payload,
            reply,
        }
    }

    #[inline]
    pub async fn serialize_and_send_reply<T: Formula + Serialize<T>>(
        self,
        payload: T,
    ) -> Result<(), TcpServerError> {
        self.send_reply(serialize_to_new_vec(payload)).await
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
    pub async fn send_reply_failed(self, fail_status: SuteraStatus) -> Result<(), TcpServerError> {
        let response = self.to_reply_failed(fail_status);
        self.reply
            .send(Response::Oneshot(response))
            .await
            .map_err(TcpServerError::CannotSendResponse)?;
        Ok(())
    }

    #[inline]
    pub async fn send_reply_bad_request(self) -> Result<(), TcpServerError> {
        self.send_reply_failed(SuteraStatus::Error(SuteraStatusError::BadRequest))
            .await
    }
    #[inline]
    pub async fn send_reply_unauthorized(self) -> Result<(), TcpServerError> {
        self.send_reply_failed(SuteraStatus::Error(SuteraStatusError::Unauthorized))
            .await
    }

    #[inline]
    pub fn to_reply(&self, payload: Vec<u8>) -> OneshotResponse {
        OneshotResponse {
            sutera_header: SuteraHeader {
                version: SCHEMA_VERSION,
            },
            sutera_status: SuteraStatus::Ok,
            oneshot_header: OneshotHeader {
                step: OneshotStep::Response,
                message_type: self.oneshot_header.message_type,
                message_id: self.oneshot_header.message_id,
            },
            payload,
        }
    }

    #[inline]
    pub fn to_reply_failed(&self, fail_status: SuteraStatus) -> OneshotResponse {
        OneshotResponse {
            sutera_header: SuteraHeader {
                version: SCHEMA_VERSION,
            },
            sutera_status: fail_status,
            oneshot_header: OneshotHeader {
                step: OneshotStep::Response,
                message_type: self.oneshot_header.message_type,
                message_id: self.oneshot_header.message_id,
            },
            payload: Vec::<u8>::new(),
        }
    }
}
