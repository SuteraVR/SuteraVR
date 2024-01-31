use std::{
    io::{self, Cursor},
    mem::size_of,
};

use bytes::{Buf, BytesMut};
use futures::{future::BoxFuture, FutureExt};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::clocking::traits::ClockingFrame;

use self::{oneshot_headers::OneshotHeader, sutera_status::SuteraStatus, traits::MessageAuthor};

pub mod buffer;
pub mod oneshot_headers;
pub mod schema_snapshot;
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

#[derive(Debug, PartialEq, Clone)]
pub enum ClockingFrameUnit {
    SuteraHeader(sutera_header::SuteraHeader),
    SuteraStatus(sutera_status::SuteraStatus),
    OneshotHeaders(oneshot_headers::OneshotHeader),
    Content(Vec<u8>),
    Unfragmented(Vec<u8>),
}

pub struct ClockingConnection<W: AsyncReadExt + AsyncWriteExt + Unpin + Send> {
    stream: W,
    buffer: BytesMut,
    author: MessageAuthor,
    context: ConnectionContext,
}
impl<W: AsyncReadExt + AsyncWriteExt + Unpin + Send> ClockingConnection<W> {
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

    pub async fn shutdown_stream(&mut self) -> io::Result<()> {
        self.stream.shutdown().await
    }

    pub async fn write_frame(
        &mut self,
        frame: &ClockingFrameUnit,
    ) -> Result<(), ClockingFramingError> {
        match frame {
            ClockingFrameUnit::SuteraHeader(header) => {
                header.write_frame(&mut self.stream, &()).await?;
            }
            ClockingFrameUnit::SuteraStatus(status) => {
                status.write_frame(&mut self.stream, &()).await?;
            }
            ClockingFrameUnit::OneshotHeaders(header) => {
                header.write_frame(&mut self.stream, &self.author).await?;
            }
            ClockingFrameUnit::Content(content) => {
                self.stream.write_u64(content.len() as u64).await?;
                self.stream.write_u64(content.len() as u64).await?;
                self.stream.write_all(content).await?;
            }
            ClockingFrameUnit::Unfragmented(content) => {
                self.stream.write_all(content).await?;
            }
        }
        self.stream.flush().await?;
        Ok(())
    }

    /// フレームを読み込みます。
    ///
    ///this function is cancellation safe.
    pub fn read_frame(
        &mut self,
    ) -> BoxFuture<'_, Result<Option<ClockingFrameUnit>, ClockingFramingError>> {
        async {
            loop {
                if let Some(frame) = self.parse_frame()? {
                    return Ok(Some(frame));
                }

                // read_buf is cancellation safe.
                if self.stream.read_buf(&mut self.buffer).await? == 0 {
                    if self.buffer.is_empty() {
                        return Ok(None);
                    } else {
                        return Err(ClockingFramingError::ConnectionReset);
                    }
                }
            }
        }
        .boxed()
    }

    #[inline]
    fn parse_frame(&mut self) -> Result<Option<ClockingFrameUnit>, ClockingFramingError> {
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
                    sutera_header::SuteraHeader::parse_frame_unchecked(&mut buf, &())
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
                self.parse_frame()
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
                    if sutera_header::SuteraHeader::parse_frame_unchecked(&mut buf, &()).is_some() {
                        self.context = ConnectionContext::None;
                        if i == 0 {
                            return self.parse_frame();
                        } else {
                            return Ok(Some(ClockingFrameUnit::Unfragmented(
                                self.buffer.copy_to_bytes(i).to_vec(),
                            )));
                        }
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
            ConnectionContext::WaitStatus => {
                buf.set_position(0);
                if let Some(status) = SuteraStatus::parse_frame(&mut buf, &()) {
                    self.context = ConnectionContext::WaitMessageType;
                    self.buffer.advance(buf.position() as usize);
                    return Ok(Some(ClockingFrameUnit::SuteraStatus(status)));
                }

                buf.set_position(0);
                let remaining = buf.remaining();
                let max_parsable_size = SuteraStatus::MAX_FRAME_SIZE;
                if remaining >= max_parsable_size {
                    self.context = ConnectionContext::Unfragmented(0);
                    return self.parse_frame();
                }

                Ok(None)
            }
            ConnectionContext::WaitMessageType => {
                buf.set_position(0);
                if let Some(header) = OneshotHeader::parse_frame(&mut buf, &self.author) {
                    self.context = ConnectionContext::WaitContent;
                    self.buffer.advance(buf.position() as usize);
                    return Ok(Some(ClockingFrameUnit::OneshotHeaders(header)));
                }

                buf.set_position(0);
                let remaining = buf.remaining();
                let max_parsable_size = OneshotHeader::MAX_FRAME_SIZE;
                if remaining >= max_parsable_size {
                    self.context = ConnectionContext::Unfragmented(0);
                    return self.parse_frame();
                }
                Ok(None)
            }
            ConnectionContext::WaitContent => {
                if buf.remaining() <= size_of::<u64>() {
                    return Ok(None);
                }

                // 送信するデータの長さを読む
                // 長さは二回同じものが出力される。
                // 同じものの場合のみ入力を受け付け、違うものの場合Contentを読んでいないと考えUnfragmentedに
                let content_length = buf.get_u64();
                let remaining = buf.remaining();
                if remaining < size_of::<u64>() {
                    return if buf.copy_to_bytes(remaining)
                        != content_length.to_be_bytes()[0..remaining]
                    {
                        // 与えられた入力が違う場合はその時点で却下
                        self.context = ConnectionContext::Unfragmented(0);
                        self.parse_frame()
                    } else {
                        // 二回目の長さが最後まで届いていないが、届いていたところまではあっている場合
                        // 続きを待つ
                        Ok(None)
                    };
                }

                let content_length_check = buf.get_u64();
                if content_length != content_length_check {
                    return Ok(None);
                }

                if buf.remaining() < content_length as usize {
                    Ok(None)
                } else {
                    let data = buf.copy_to_bytes(content_length as usize);
                    self.buffer.advance(buf.position() as usize);
                    self.context = ConnectionContext::None;
                    Ok(Some(ClockingFrameUnit::Content(data.to_vec())))
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::clocking::oneshot_headers::OneshotStep;
    use crate::clocking::oneshot_headers::OneshotTypes;
    use crate::clocking::OneshotHeader;
    use crate::clocking::SuteraStatus;
    use rstest::*;
    use std::io::{Cursor, Write};

    use crate::clocking::traits::test_util::encode;
    use crate::clocking::{ClockingConnection, ClockingFrameUnit};
    use crate::{clocking::sutera_header::SuteraHeader, messaging::version::Version};
    use pretty_assertions::assert_eq;

    use super::traits::MessageAuthor;

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

    #[rstest(
        author => [MessageAuthor::Client, MessageAuthor::Server],
        inject => [&[0x01, 0x02, 0x03], b"LongPayloooooooooooooad", &[0x0f]],
        ::trace
    )]
    #[tokio::test]
    async fn read_with_unfragmented_size(author: MessageAuthor, inject: &[u8]) {
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
        vec.write_all(inject).unwrap();
        vec.write_all(&payload[..]).unwrap();

        vec.set_position(0);
        let mut connection = ClockingConnection::new(&mut vec, author);
        assert_eq!(
            connection.read_frame().await.unwrap(),
            Some(ClockingFrameUnit::SuteraHeader(header.clone()))
        );
        assert_eq!(
            connection.read_frame().await.unwrap(),
            Some(ClockingFrameUnit::Unfragmented(inject.into()))
        );
        assert_eq!(
            connection.read_frame().await.unwrap(),
            Some(ClockingFrameUnit::SuteraHeader(header.clone()))
        );
    }

    #[rstest]
    #[case::client(MessageAuthor::Client)]
    #[case::server(MessageAuthor::Server)]
    #[tokio::test]
    async fn read_all_frame(#[case] author: MessageAuthor) {
        let mut vec = Cursor::new(Vec::<u8>::new());

        let header = SuteraHeader {
            version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
        };

        let status = SuteraStatus::Ok;

        let oneshot_header = OneshotHeader {
            step: match author {
                MessageAuthor::Client => OneshotStep::Request,
                MessageAuthor::Server => OneshotStep::Response,
            },
            message_id: 0x1234,
            message_type: OneshotTypes::Authentication_Login_Pull,
        };

        let payload = b"Wao!";

        vec.write_all(&encode(&header, &()).await).unwrap();

        if author == MessageAuthor::Server {
            vec.write_all(&encode(&status, &()).await).unwrap();
        }

        vec.write_all(&encode(&oneshot_header, &author).await)
            .unwrap();

        vec.write_all(&(payload.len() as u64).to_be_bytes())
            .unwrap();
        vec.write_all(&(payload.len() as u64).to_be_bytes())
            .unwrap();
        vec.write_all(&payload[..]).unwrap();

        vec.set_position(0);
        let mut connection = ClockingConnection::new(&mut vec, author);
        assert_eq!(
            connection.read_frame().await.unwrap(),
            Some(ClockingFrameUnit::SuteraHeader(header))
        );
        if author == MessageAuthor::Server {
            assert_eq!(
                connection.read_frame().await.unwrap(),
                Some(ClockingFrameUnit::SuteraStatus(status))
            );
        }
        assert_eq!(
            connection.read_frame().await.unwrap(),
            Some(ClockingFrameUnit::OneshotHeaders(oneshot_header))
        );
        assert_eq!(
            connection.read_frame().await.unwrap(),
            Some(ClockingFrameUnit::Content(payload.into()))
        );
    }
}
