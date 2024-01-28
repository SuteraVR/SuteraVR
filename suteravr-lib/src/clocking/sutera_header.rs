use std::mem::size_of;

use bytes::Buf;
use tokio::io::AsyncWriteExt;

use crate::messaging::version::Version;

use super::traits::ClockingFrame;

#[derive(Debug, PartialEq)]
pub struct SuteraHeader {
    pub message_length: u64,
    pub version: Version,
}

impl SuteraHeader {
    const PREFIX: &'static [u8] = b"SuteraVR";
}
impl ClockingFrame for SuteraHeader {
    type Context = ();
    const MINIMAL_FRAME_SIZE: usize =
        Self::PREFIX.len() + size_of::<u64>() + Version::MINIMAL_FRAME_SIZE;

    async fn parse_frame_unchecked(
        cursor: &mut std::io::Cursor<&[u8]>,
        _ctx: &Self::Context,
    ) -> Option<Self> {
        if Self::PREFIX != cursor.copy_to_bytes(Self::PREFIX.len()) {
            return None;
        }
        let message_length = cursor.get_u64();
        let version = Version::parse_frame_unchecked(cursor, &()).await?;
        Some(Self {
            message_length,
            version,
        })
    }

    async fn write_frame<W: tokio::io::AsyncWriteExt + Unpin>(
        &self,
        stream: &mut tokio::io::BufWriter<W>,
        _ctx: &Self::Context,
    ) -> std::io::Result<()> {
        stream.write_all(Self::PREFIX).await?;
        stream.write_u64(self.message_length).await?;
        self.version.write_frame(stream, _ctx).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::clocking::traits::test_util::{decode, encode, test_clockingframe_reflective};

    use super::*;

    #[tokio::test]
    async fn clockingframe_suteraheader() {
        test_clockingframe_reflective(
            SuteraHeader {
                message_length: 194u64,
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                },
            },
            (),
        )
        .await
    }

    #[tokio::test]
    async fn clockingframe_suteraheader_prefixerr() {
        let mut encoded = encode(
            &SuteraHeader {
                message_length: 194u64,
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                },
            },
            &(),
        )
        .await;
        encoded[0] = b'T';
        encoded[1] = b'e';
        encoded[2] = b's';
        encoded[3] = b'u';
        encoded[4] = b'r';
        encoded[5] = b'a';

        let (decoded, cursor) = decode::<SuteraHeader>(&encoded, &()).await;
        assert_eq!(decoded, None);
        assert_eq!(cursor.position() as usize, SuteraHeader::PREFIX.len());
    }
}