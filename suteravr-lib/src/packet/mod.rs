//! SuteRPCの通信に必要な情報を扱うモジュール

pub mod header;
pub mod request_type;
pub mod semver;

use alkahest::advanced::BareFormula;
use alkahest::serialize_to_vec;
use alkahest::Formula;
use alkahest::SerializeRef;
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
use crate::typing::SizedForBinary;

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

pub struct OneshotResponsePayload<
    T: OneshotResponseMarker + Formula + BareFormula + SerializeRef<T>,
> {
    pub header: ResponseHeader,
    pub payload: T,
}

impl<T: OneshotResponseMarker + Formula + BareFormula + SerializeRef<T>>
    From<OneshotResponsePayload<T>> for Vec<u8>
{
    /// [`OneshotRequestPayload`]をバイナリデータに変換します。
    ///
    /// # Example
    /// ```
    /// use suteravr_lib::packet::{
    ///   RequestType,
    ///   ResponseHeader,
    ///   OneshotResponsePayload,
    /// };
    /// use suteravr_lib::schema_oneshot::{
    ///   responses,
    ///   OneshotVariants,
    /// };
    /// use suteravr_lib::typing::id::RequestIdentifier;
    /// use suteravr_lib::semver::Semver;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let payload: OneshotResponsePayload<responses::GetVersion>
    ///   = OneshotResponsePayload {
    ///     header: ResponseHeader {
    ///       schema_version: "0.1.0".into(),
    ///       request_id: RequestIdentifier(1),
    ///       request_type: RequestType::Oneshot(OneshotVariants::GetVersion),
    ///     },
    ///     payload: responses::GetVersion {
    ///       version: "1.12.2".into(),
    ///     },
    /// };
    ///
    /// let binary: Vec<u8> = payload.into();
    /// assert_eq!(
    ///   binary,
    ///   vec![
    ///     0, 1, 0,                  // schema_version
    ///     0, 0, 0, 0, 0, 0, 0, 1,   // request_id
    ///     1, 0,                     // request_type (GetVersion)
    ///     2, 12, 1,                 // payload
    ///   ]
    /// );
    /// # Ok(())
    /// # }
    ///
    fn from(val: OneshotResponsePayload<T>) -> Self {
        let mut buf = vec![0u8; ResponseHeader::SIZE];

        // Write ResponseHeader::schmea_version
        {
            let write_place: &mut [u8; 3] = (&mut buf[0..ResponseHeader::REQUEST_ID_OFFSET])
                .try_into()
                .unwrap();
            val.header.schema_version.write(write_place);
        }

        // Write ResponseHeader::request_id
        {
            let write_place: &mut [u8; 8] = (&mut buf
                [ResponseHeader::REQUEST_ID_OFFSET..ResponseHeader::REQUEST_TYPE_OFFSET])
                .try_into()
                .unwrap();
            write_place.copy_from_slice(&val.header.request_id.0.to_be_bytes());
        }

        // Write ResponseHeader::request_type
        {
            let write_place: &mut [u8; 2] = (&mut buf
                [ResponseHeader::REQUEST_TYPE_OFFSET..ResponseHeader::SIZE])
                .try_into()
                .unwrap();
            val.header.request_type.write(write_place);
        }

        let mut payload = Vec::new();
        // Write Payload
        serialize_to_vec::<T, _>(&val.payload, &mut payload);

        buf.append(&mut payload);

        buf
    }
}
