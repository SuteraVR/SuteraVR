//! SuteRPCのヘッダーを扱うモジュール

use thiserror::Error;

use super::request_type::{RequestType, RequestTypeDeserializeError};
use crate::{
    semver::Semver,
    typing::{
        id::{InstanceIdentifier, PlayerIdentifier, RequestIdentifier},
        SizedForBinary,
    },
};

/// SuteRPCの全リクエストにつけられるヘッダー
///
/// ([`sender`], [`request_id`])の順序列は、インスタンスに対して一意にならなければなりません。
/// [`sender`]は、[`PlayerIdentifier`] (クロッキングサーバー内で一意)であるため、
/// ([`sender`], [`request_id`])の順序列は、クロッキングサーバー内で一意になります。
///
/// # Serde
///
///
/// [`sender`]: RequestHeader::sender
/// [`request_id`]: RequestHeader::request_id
#[derive(Debug, Eq, PartialEq)]
pub struct RequestHeader {
    /// スキーマのバージョン
    pub schema_version: Semver,
    /// リクエストのID
    pub request_id: RequestIdentifier,
    /// リクエストの送信者
    pub sender: PlayerIdentifier,
    /// リクエストの種類と、そのバリアント
    pub request_type: RequestType,
}
impl SizedForBinary for RequestHeader {
    const SIZE: usize =
        Semver::SIZE + RequestIdentifier::SIZE + PlayerIdentifier::SIZE + RequestType::SIZE;
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RequestParseError {
    /// リクエストが、ヘッダーデータのサイズよりも短い
    #[error("Request is too short to include header.")]
    TooShort,
    #[error(transparent)]
    BadRequestType(#[from] RequestTypeDeserializeError),
}

impl RequestHeader {
    const REQUEST_ID_OFFSET: usize = Semver::SIZE;
    const SENDER_OFFSET: usize = Self::REQUEST_ID_OFFSET + RequestIdentifier::SIZE;
    const REQUEST_TYPE_OFFSET: usize = Self::SENDER_OFFSET + PlayerIdentifier::SIZE;

    /// バイナリデータから [`RequestHeader`] をパースします。
    ///
    /// # Example
    /// ```
    /// use suteravr_lib::packet::header::RequestHeader;
    /// use suteravr_lib::typing::id::{InstanceIdentifier, PlayerIdentifier, RequestIdentifier};
    /// use suteravr_lib::packet::request_type::RequestType;
    /// use suteravr_lib::schema::oneshot::OneshotVariants;
    /// use suteravr_lib::semver::Semver;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let payload = vec![
    ///   0, 1, 0,                  // schema version
    ///   0, 0, 0, 0, 0, 0, 0, 1,   // request_id
    ///   4,                        // instance
    ///   1,                        // player
    ///   0, 0,                     // request type (OneshotRequest, GetVersion)
    ///   1, 2, 3, 4, 5, 6, 7, 8, 9 // payload
    /// ];
    ///
    /// let (header, payload) = RequestHeader::parse_binary(&payload)?;
    /// assert_eq!(
    ///   header,
    ///   RequestHeader {
    ///     schema_version: Semver { major: 0, minor: 1, patch: 0 },
    ///     request_id: RequestIdentifier(1),
    ///     sender: PlayerIdentifier(InstanceIdentifier(4), 1),
    ///     request_type: RequestType::Oneshot(OneshotVariants::GetVersion),
    ///   }
    /// );
    /// assert_eq!(payload, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_binary(payload: &[u8]) -> Result<(Self, &[u8]), RequestParseError> {
        if payload.len() < Self::SIZE {
            return Err(RequestParseError::TooShort);
        }

        // .try_into は、&[u8] -> [u8; N]に変換する。
        // &[u8].len() == N であればこの変換は必ず成功するので、unwrap() してよい。
        let schema_version = {
            let part: [u8; 3] = payload[0..Self::REQUEST_ID_OFFSET].try_into().unwrap();
            Semver::from(part)
        };
        let request_id = RequestIdentifier(u64::from_be_bytes(
            payload[Self::REQUEST_ID_OFFSET..Self::SENDER_OFFSET]
                .try_into()
                .unwrap(),
        ));
        let instance = InstanceIdentifier(payload[Self::SENDER_OFFSET]);
        let sender = PlayerIdentifier(instance, payload[Self::SENDER_OFFSET + 1]);

        let request_type = {
            let part: [u8; 2] = payload[Self::REQUEST_TYPE_OFFSET..Self::SIZE]
                .try_into()
                .unwrap();
            RequestType::try_from(part).map_err(RequestParseError::BadRequestType)?
        };

        Ok((
            Self {
                schema_version,
                request_id,
                sender,
                request_type,
            },
            &payload[Self::SIZE..],
        ))
    }
}

/// SuteRPCの全レスポンスにつけられるヘッダー
///
#[derive(Debug)]
pub struct ResponseHeader {
    /// リクエストのID
    /// [`request_type`] が [`RequestType::Oneshot`] の場合、これは [`RequestHeader::request_id`] と一致する必要があります。
    /// そうではない場合、クライアントはこの値を無視します。
    ///
    /// [`request_type`]: ResponseHeader::request_type
    pub request_id: u64,
    /// リクエストの種類と、そのバリアント
    pub request_type: RequestType,
}
