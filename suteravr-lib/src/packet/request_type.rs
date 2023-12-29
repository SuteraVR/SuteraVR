//! SuteRPC上のリクエストの種類を扱うモジュール

use num_traits::FromPrimitive;
use thiserror::Error;

use crate::schema_oneshot::OneshotVariants;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RequestTypeSerializeError {
    #[error("Length of bytes must be 2, but got {0}")]
    UnexpectedLength(usize),
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RequestTypeDeserializeError {
    #[error("Length of bytes must be 2, but got {0}")]
    UnexpectedLength(usize),
    #[error("Unknown request type: {0}")]
    UnknownRequestType(u8),
    #[error("Unknown variant of oneshot: {0}")]
    UnknownVariantOfOneshot(u8),
}

/// SuteRPC上のリクエストの種類を表すenum  
/// 形式とバリアントがセットで提供される
///
/// 現在はワンショットリクエストのみが存在しています
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RequestType {
    Oneshot(OneshotVariants) = 0,
}

///
/// 各種SuteRPCのバリアントから、[`RequestType`]に変換します
///
/// # Example
/// ```
/// use suteravr_lib::packet::request_type::RequestType;
/// use suteravr_lib::schema_oneshot::OneshotVariants;
///
/// let req_type: RequestType = OneshotVariants::GetVersion.into();
/// assert_eq!(req_type, RequestType::Oneshot(OneshotVariants::GetVersion));
/// ```
impl From<OneshotVariants> for RequestType {
    fn from(v: OneshotVariants) -> Self {
        Self::Oneshot(v)
    }
}

impl RequestType {
    /// [`RequestType`]を`u8;2`のバイト列に書きこみます。
    ///
    /// # Example
    /// ```
    /// use suteravr_lib::packet::request_type::RequestType;
    /// use suteravr_lib::schema_oneshot::OneshotVariants;
    ///
    /// let mut buf = vec![0, 0, 0, 0];
    ///
    /// assert_eq!(OneshotVariants::RequestPlayerAuth as u8, 2);
    ///
    /// let req_type: RequestType = OneshotVariants::RequestPlayerAuth.into();
    /// req_type.try_write(&mut buf[1..3]).unwrap();
    /// assert_eq!(buf, vec![0, 0, 2, 0]);
    /// ```
    pub fn try_write(&self, buf: &mut [u8]) -> Result<(), RequestTypeSerializeError> {
        if buf.len() != 2 {
            return Err(RequestTypeSerializeError::UnexpectedLength(buf.len()));
        }
        match self {
            Self::Oneshot(v) => {
                buf[0] = 0;
                buf[1] = *v as u8;
            }
        }
        Ok(())
    }
}

impl TryFrom<&[u8]> for RequestType {
    type Error = RequestTypeDeserializeError;
    ///
    /// `u8;2`のバイト列から[`RequestType`]を読み込みます。
    ///
    /// # Example
    /// ```
    /// use suteravr_lib::packet::request_type::RequestType;
    /// use suteravr_lib::schema_oneshot::OneshotVariants;
    ///
    /// let buf = vec![0, 2];
    /// let req_type = RequestType::try_from(&buf[..]).unwrap();
    /// assert_eq!(req_type, RequestType::Oneshot(OneshotVariants::RequestPlayerAuth));
    /// ```
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        if buf.len() != 2 {
            return Err(RequestTypeDeserializeError::UnexpectedLength(buf.len()));
        }
        match buf[0] {
            0 => FromPrimitive::from_u8(buf[1])
                .ok_or(RequestTypeDeserializeError::UnknownVariantOfOneshot(buf[1]))
                .map(OneshotVariants::into),
            _ => Err(RequestTypeDeserializeError::UnknownRequestType(buf[0])),
        }
    }
}
