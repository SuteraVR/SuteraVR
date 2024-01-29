use std::mem::size_of;

use bytes::Buf;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::clocking::traits::ClockingFrame;

#[derive(Debug, PartialEq, Clone)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ClockingFrame for Version {
    type Context = ();
    const MIN_FRAME_SIZE: usize = size_of::<[u16; 3]>();

    async fn parse_frame_unchecked(
        cursor: &mut std::io::Cursor<&[u8]>,
        _context: &Self::Context,
    ) -> Option<Self> {
        let major = cursor.get_u16();
        let minor = cursor.get_u16();
        let patch = cursor.get_u16();
        Some(Self {
            major,
            minor,
            patch,
        })
    }

    async fn write_frame<W: AsyncWriteExt + Unpin>(
        &self,
        stream: &mut BufWriter<W>,
        _context: &Self::Context,
    ) -> std::io::Result<()> {
        stream.write_u16(self.major).await?;
        stream.write_u16(self.minor).await?;
        stream.write_u16(self.patch).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        clocking::traits::test_util::test_clockingframe_reflective, messaging::version::Version,
    };

    #[tokio::test]
    async fn clockingframe_version() {
        test_clockingframe_reflective(
            Version {
                major: 1,
                minor: 2,
                patch: 3,
            },
            (),
        )
        .await;
    }
}
