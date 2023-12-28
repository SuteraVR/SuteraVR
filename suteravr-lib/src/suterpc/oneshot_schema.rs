use alkahest::alkahest;
use async_trait::async_trait;

use super::{Oneshot, OneshotRequest, OneshotResult, Sender};

pub(crate) enum OneshotVariants {
    GetVersion = 1,
}

#[alkahest(Formula, SerializeRef, Deserialize)]
pub(crate) struct GetVersionRequest {}
#[async_trait]
impl OneshotRequest<GetVersionResponse> for GetVersionRequest {
    async fn send<T: Sender + Send>(self, server: T) -> OneshotResult<GetVersionResponse> {
        server
            .send_oneshot::<GetVersion>(OneshotVariants::GetVersion, self)
            .await
    }
}

#[alkahest(Formula, SerializeRef, Deserialize)]
pub(crate) struct GetVersionResponse {
    version: u64,
}

pub struct GetVersion {}
impl Oneshot<'_> for GetVersion {
    type Request = GetVersionRequest;
    type Response = GetVersionResponse;
}
