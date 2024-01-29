use bytes::Buf;
use enum_map::enum_map;
use enum_map::Enum;
use enum_map::EnumMap;
use once_cell::sync::Lazy;

use crate::util::search_from_enum;

use super::traits::ClockingFrame;

#[derive(Enum, PartialEq, Debug, Clone, Copy)]
pub enum SuteraStatusError {
    SchemaVersionNotSupported,
    BadRequest,
    Unimplemented,
    Unauthorized,
    AuthenticationHasBeenExpired,
    Forbidden,
    YouAreNotInInstance,
}
static SUTERA_STATUS_ERROR_MAP: Lazy<EnumMap<SuteraStatusError, [u8; 3]>> = Lazy::new(|| {
    enum_map! {
        SuteraStatusError::SchemaVersionNotSupported    => [0x10, 0x00, 0x00],
        SuteraStatusError::BadRequest                   => [0x10, 0x02, 0x00],
        SuteraStatusError::Unimplemented                => [0x10, 0x02, 0x01],
        SuteraStatusError::Unauthorized                 => [0x20, 0x00, 0x00],
        SuteraStatusError::AuthenticationHasBeenExpired => [0x20, 0x00, 0x01],
        SuteraStatusError::Forbidden                    => [0x20, 0x01, 0x00],
        SuteraStatusError::YouAreNotInInstance          => [0x20, 0x02, 0x00],
    }
});

#[derive(Enum, PartialEq, Debug, Clone, Copy)]
pub enum SuteraStatusWarning {
    SchemaVersionNotExactlyMatched,
}
static SUTERA_STATUS_WARNING_MAP: Lazy<EnumMap<SuteraStatusWarning, [u8; 3]>> = Lazy::new(|| {
    enum_map! {
        SuteraStatusWarning::SchemaVersionNotExactlyMatched => [0x10, 0x00, 0x00],
    }
});

#[derive(Debug, PartialEq)]
pub enum SuteraStatus {
    Ok,
    Warning(SuteraStatusWarning),
    Error(SuteraStatusError),
}

impl ClockingFrame for SuteraStatus {
    type Context = ();

    const MIN_FRAME_SIZE: usize = 1;
    const MAX_FRAME_SIZE: usize = 4;

    async fn parse_frame_unchecked(
        cursor: &mut std::io::Cursor<&[u8]>,
        _ctx: &Self::Context,
    ) -> Option<Self> {
        match cursor.copy_to_bytes(1)[0] {
            0x00 => Some(Self::Ok),
            0x01 => {
                if cursor.remaining() < 3 {
                    return None;
                }
                let kind = [cursor.get_u8(), cursor.get_u8(), cursor.get_u8()];
                search_from_enum(*SUTERA_STATUS_WARNING_MAP, &kind).map(Self::Warning)
            }
            0x02 => {
                if cursor.remaining() < 3 {
                    return None;
                }
                let kind = [cursor.get_u8(), cursor.get_u8(), cursor.get_u8()];
                search_from_enum(*SUTERA_STATUS_ERROR_MAP, &kind).map(Self::Error)
            }
            _ => None,
        }
    }

    async fn write_frame<W: tokio::io::AsyncWriteExt + Unpin>(
        &self,
        stream: &mut W,
        _ctx: &Self::Context,
    ) -> std::io::Result<()> {
        match self {
            Self::Ok => stream.write_all(&[0x00]).await?,
            Self::Warning(w) => {
                stream.write_all(&[0x01]).await?;
                stream.write_all(&SUTERA_STATUS_WARNING_MAP[*w]).await?;
            }
            Self::Error(e) => {
                stream.write_all(&[0x02]).await?;
                stream.write_all(&SUTERA_STATUS_ERROR_MAP[*e]).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::clocking::traits::test_util::{
        test_clockingframe_mustfail, test_clockingframe_reflective,
    };

    use super::*;

    #[tokio::test]
    async fn clockingframe_sutera_status() {
        test_clockingframe_reflective(SuteraStatus::Ok, ()).await;
        test_clockingframe_reflective(
            SuteraStatus::Warning(SuteraStatusWarning::SchemaVersionNotExactlyMatched),
            (),
        )
        .await;
        test_clockingframe_reflective(SuteraStatus::Error(SuteraStatusError::BadRequest), ()).await;
        test_clockingframe_reflective(
            SuteraStatus::Error(SuteraStatusError::SchemaVersionNotSupported),
            (),
        )
        .await;
    }

    #[tokio::test]
    async fn clockingframe_sutera_status_prefixerr() {
        test_clockingframe_mustfail::<SuteraStatus>(&[0x02, 0x01, 0x00], &(), Some(1)).await;
        test_clockingframe_mustfail::<SuteraStatus>(&[0x03, 0x00, 0x00, 0x00], &(), Some(1)).await;
        test_clockingframe_mustfail::<SuteraStatus>(&[0x03, 0x01, 0x00, 0x00], &(), Some(1)).await;
    }

    #[tokio::test]
    async fn clockingframe_sutera_status_unexists() {
        test_clockingframe_mustfail::<SuteraStatus>(&[0x02, 0xFF, 0xFF, 0xFF], &(), Some(4)).await;
    }
}
