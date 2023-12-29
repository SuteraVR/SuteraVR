//! SuteRPCの通信に必要な情報を扱うモジュール

pub mod header;
pub mod request_type;
pub mod semver;

use async_trait::async_trait;
#[doc(inline)]
pub use header::RequestHeader;
#[doc(inline)]
pub use header::ResponseHeader;
#[doc(inline)]
pub use request_type::RequestType;
#[doc(inline)]
pub use semver::Semver;

use crate::suterpc::Oneshot;
use crate::suterpc::OneshotRequestMarker;
use crate::suterpc::OneshotResponseMarker;

#[async_trait]
pub trait OneshotImplementer<'de, T: Oneshot<'de>> {
    type Error;
    async fn handle(
        &self,
        req: OneshotRequestPayload<T::Request>,
    ) -> Result<OneshotResponsePayload<T::Response>, Self::Error>;
}

pub struct OneshotRequestPayload<T: OneshotRequestMarker> {
    pub header: RequestHeader,
    pub payload: T,
}

pub struct OneshotResponsePayload<T: OneshotResponseMarker> {
    pub header: ResponseHeader,
    pub payload: T,
}
