pub(crate) mod macro_impl;

use alkahest::{advanced::BareFormula, Deserialize, Formula, SerializeRef};
use async_trait::async_trait;

use crate::schema::oneshot::OneshotVariants;

// TODO: OneshotError type is need.
pub(crate) type OneshotResult<Response> = Result<Response, ()>;

#[async_trait]
pub(crate) trait Sender {
    async fn send_oneshot<'de, T: Oneshot<'de>>(
        &self,
        variant: OneshotVariants,
        payload: T::Request,
    ) -> OneshotResult<T::Response>;
}

#[async_trait]
pub(crate) trait OneshotRequest<Response> {
    async fn send<T: Sender + Send>(self, server: T) -> OneshotResult<Response>;
}

pub(crate) trait Oneshot<'de> {
    type Request: Send
        + Formula
        + BareFormula
        + SerializeRef<Self::Request>
        + Deserialize<'de, Self::Request>
        + OneshotRequest<Self::Response>;
    type Response: Send
        + Formula
        + BareFormula
        + SerializeRef<Self::Response>
        + Deserialize<'de, Self::Response>;
}
