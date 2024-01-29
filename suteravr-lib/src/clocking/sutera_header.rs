use bytes::Buf;
use tokio::io::AsyncWriteExt;

use crate::messaging::version::Version;

use super::traits::ClockingFrame;

#[derive(Debug, PartialEq, Clone)]
pub struct SuteraHeader {
    pub version: Version,
}

impl SuteraHeader {
    const PREFIX: &'static [u8] = b"SuteraVR";
}
impl ClockingFrame for SuteraHeader {
    type Context = ();
    const MIN_FRAME_SIZE: usize = Self::PREFIX.len() + Version::MIN_FRAME_SIZE;

    async fn parse_frame_unchecked(
        cursor: &mut std::io::Cursor<&[u8]>,
        _ctx: &Self::Context,
    ) -> Option<Self> {
        if Self::PREFIX != cursor.copy_to_bytes(Self::PREFIX.len()) {
            return None;
        }
        let version = Version::parse_frame_unchecked(cursor, &()).await?;
        Some(Self { version })
    }

    async fn write_frame<W: tokio::io::AsyncWriteExt + Unpin>(
        &self,
        stream: &mut tokio::io::BufWriter<W>,
        _ctx: &Self::Context,
    ) -> std::io::Result<()> {
        stream.write_all(Self::PREFIX).await?;
        self.version.write_frame(stream, _ctx).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::clocking::traits::test_util::{
        encode, test_clockingframe_mustfail, test_clockingframe_reflective,
    };

    use super::*;

    #[tokio::test]
    async fn clockingframe_suteraheader() {
        test_clockingframe_reflective(
            SuteraHeader {
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

        test_clockingframe_mustfail::<SuteraHeader>(
            &encoded,
            &(),
            Some(SuteraHeader::PREFIX.len()),
        )
        .await;
    }
}
