mod oneshot_schema;

use alkahest::{advanced::BareFormula, Deserialize, Formula, SerializeRef};
use async_trait::async_trait;

use self::oneshot_schema::OneshotVariants;

#[async_trait]
trait Server {
    async fn send_oneshot<'de, T: Oneshot<'de>>(
        &self,
        variant: OneshotVariants,
        payload: T::Request,
    ) -> T::Response;
}

#[async_trait]
trait Request<Response> {
    async fn send(&self, server: &impl Server) -> Response;
}

trait Oneshot<'de> {
    type Request: Formula
        + BareFormula
        + SerializeRef<Self::Request>
        + Deserialize<'de, Self::Request>
        + Request<Self::Response>;
    type Response: Formula
        + BareFormula
        + SerializeRef<Self::Response>
        + Deserialize<'de, Self::Response>;
}
