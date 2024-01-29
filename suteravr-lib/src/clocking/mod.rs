use std::io::Cursor;

use async_recursion::async_recursion;
use bytes::{Buf, BytesMut};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::clocking::traits::ClockingFrame;

use self::{oneshot_headers::OneshotHeader, traits::MessageAuthor};

pub mod oneshot_headers;
pub mod sutera_header;
pub mod sutera_status;
pub mod traits;

#[derive(Error, Debug)]
pub enum ClockingFramingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Connection reset by peer")]
    ConnectionReset,
}

enum ConnectionContext {
    None,
    Unfragmented(usize),
    WaitStatus,
    WaitMessageType,
    WaitContent,
}

#[derive(Debug, PartialEq)]
pub enum ClockingFrameUnit {
    SuteraHeader(sutera_header::SuteraHeader),
    SuteraStatus(sutera_status::SuteraStatus),
    OneshotHeaders(oneshot_headers::OneshotHeader),
    Content(Vec<u8>),
    Unfragmented(Vec<u8>),
}

pub struct ClockingConnection<W: AsyncReadExt + AsyncWriteExt + Unpin> {
    stream: W,
    buffer: BytesMut,
    author: MessageAuthor,
    context: ConnectionContext,
}
impl<W: AsyncReadExt + AsyncWriteExt + Unpin> ClockingConnection<W> {
    /// 既存のストリームから新しいClockingConnectionを作成します。
    ///
    /// **authorには、名前の通り「メッセージの送信者」が格納されることに注意してください。**
    /// たとえば、サーバー側で動いている場合は、[`MessageAuthor::Client`]を`author`に指定する必要があります。`
    pub fn new(stream: W, author: MessageAuthor) -> Self {
        Self {
            stream,
            author,
            buffer: BytesMut::with_capacity(4096),
            context: ConnectionContext::None,
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<ClockingFrameUnit>, ClockingFramingError> {
        loop {
            if let Some(frame) = self.parse_frame().await? {
                return Ok(Some(frame));
            }

            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(ClockingFramingError::ConnectionReset);
                }
            }
        }
    }

    #[async_recursion(?Send)]
    async fn parse_frame(&mut self) -> Result<Option<ClockingFrameUnit>, ClockingFramingError> {
        let mut buf = Cursor::new(&self.buffer[..]);
        match self.context {
            ConnectionContext::None => {
                let remaining = buf.remaining();

                // SuteraHeader (通信のはじまりの目印) を探す
                // この時点で、SuteraHeaderの最小サイズよりも小さい場合は、次のバッファも読む
                if remaining < sutera_header::SuteraHeader::MIN_FRAME_SIZE {
                    return Ok(None);
                }

                // 先頭からSuteraHeaderが成立していれば文句なしでOK
                buf.set_position(0);
                if let Some(header) =
                    sutera_header::SuteraHeader::parse_frame_unchecked(&mut buf, &()).await
                {
                    self.buffer.advance(buf.position() as usize);
                    self.context = match self.author {
                        MessageAuthor::Server => ConnectionContext::WaitStatus,
                        MessageAuthor::Client => ConnectionContext::WaitMessageType,
                    };
                    return Ok(Some(ClockingFrameUnit::SuteraHeader(header)));
                }

                if remaining < sutera_header::SuteraHeader::MAX_FRAME_SIZE {
                    return Ok(None);
                }

                // 処理がここまで流れている時点で、
                // 最後にフレームが成立してから直ちにヘッダーが来ていないから何かがおかしい
                //
                // ただ、一応次にどこかでSuteraHeaderが来るかもしれないので、
                // バッファのどこかにSuteraHeaderを検知できたらそれまでのところをUnfragmentedとする
                self.context = ConnectionContext::Unfragmented(1);
                self.parse_frame().await
            }
            ConnectionContext::Unfragmented(checked_length) => {
                let remaining = buf.remaining();

                // この時点で、SuteraHeaderの最小サイズよりも小さい場合は、次のバッファも読む
                if remaining < sutera_header::SuteraHeader::MIN_FRAME_SIZE {
                    return Ok(None);
                }

                // 新しく増えた領域にSuteraHeaderが存在しないか確認する
                let last_possible_index = remaining - sutera_header::SuteraHeader::MIN_FRAME_SIZE;
                for i in checked_length..=last_possible_index {
                    buf.set_position(i as u64);
                    if (sutera_header::SuteraHeader::parse_frame_unchecked(&mut buf, &()).await)
                        .is_some()
                    {
                        self.context = ConnectionContext::None;
                        return Ok(Some(ClockingFrameUnit::Unfragmented(
                            self.buffer.copy_to_bytes(i).to_vec(),
                        )));
                    }
                }

                // 認識されていない状態でバッファが増えつづけると危険なので、
                // 定期的にUnfragmentedとして処理する
                if remaining > 1024 {
                    self.context = ConnectionContext::Unfragmented(0);
                    Ok(Some(ClockingFrameUnit::Unfragmented(
                        self.buffer.copy_to_bytes(1024).to_vec(),
                    )))
                } else {
                    self.context = ConnectionContext::Unfragmented(checked_length + 1);
                    Ok(None)
                }
            }
            ConnectionContext::WaitStatus => todo!(),
            ConnectionContext::WaitMessageType => {
                buf.set_position(0);
                if let Some(header) = OneshotHeader::parse_frame(&mut buf, &self.author).await {
                    self.context = ConnectionContext::WaitContent;
                    return Ok(Some(ClockingFrameUnit::OneshotHeaders(header)));
                }

                buf.set_position(0);
                let remaining = buf.remaining();
                let max_parsable_size = OneshotHeader::MAX_FRAME_SIZE;
                if remaining >= max_parsable_size {
                    self.context = ConnectionContext::Unfragmented(0);
                    return self.parse_frame().await;
                }
                Ok(None)
            }
            ConnectionContext::WaitContent => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Write};

    use crate::clocking::traits::test_util::encode;
    use crate::clocking::{ClockingConnection, ClockingFrameUnit};
    use crate::{clocking::sutera_header::SuteraHeader, messaging::version::Version};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn read_header_chunk() {
        let mut vec = Cursor::new(Vec::<u8>::new());
        let header = SuteraHeader {
            version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
        };
        let payload = encode(&header, &()).await;
        vec.write_all(&payload[..]).unwrap();

        vec.set_position(0);
        let mut connection =
            ClockingConnection::new(&mut vec, crate::clocking::traits::MessageAuthor::Client);
        assert_eq!(
            connection.read_frame().await.unwrap(),
            Some(ClockingFrameUnit::SuteraHeader(header))
        );
    }

    #[tokio::test]
    async fn read_with_unfragmented_size() {
        let mut vec = Cursor::new(Vec::<u8>::new());
        let header = SuteraHeader {
            version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
        };
        let payload = encode(&header, &()).await;
        vec.write_all(&payload[..]).unwrap();
        vec.write_all(&[0x01, 0x02, 0x03]).unwrap();
        vec.write_all(&payload[..]).unwrap();

        vec.set_position(0);
        let mut connection =
            ClockingConnection::new(&mut vec, crate::clocking::traits::MessageAuthor::Client);
        assert_eq!(
            connection.read_frame().await.unwrap(),
            Some(ClockingFrameUnit::SuteraHeader(header.clone()))
        );
        assert_eq!(
            connection.read_frame().await.unwrap(),
            Some(ClockingFrameUnit::Unfragmented(vec![0x01, 0x02, 0x03]))
        );
        assert_eq!(
            connection.read_frame().await.unwrap(),
            Some(ClockingFrameUnit::SuteraHeader(header.clone()))
        );
    }
}
