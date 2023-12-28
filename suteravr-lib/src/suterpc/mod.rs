mod oneshot_schema;

use alkahest::{advanced::BareFormula, Deserialize, Formula, SerializeRef};
use async_trait::async_trait;

use self::oneshot_schema::OneshotVariants;

// TODO: OneshotError type is need.
type OneshotResult<Response> = Result<Response, ()>;

#[async_trait]
trait Sender {
    async fn send_oneshot<'de, T: Oneshot<'de>>(
        &self,
        variant: OneshotVariants,
        payload: T::Request,
    ) -> OneshotResult<T::Response>;
}

#[async_trait]
trait OneshotRequest<Response> {
    async fn send<T: Sender + Send>(self, server: T) -> OneshotResult<Response>;
}

trait Oneshot<'de> {
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
