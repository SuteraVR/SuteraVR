use std::io::Cursor;

use bytes::{Buf, BytesMut};
use thiserror::Error;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::clocking::traits::ClockingFrame;

use self::traits::MessageAuthor;

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
    WaitStatus(usize),
    WaitMessageType(usize),
    WaitContent(usize),
}

pub enum ClockingFrameUnit {
    SuteraHeader(sutera_header::SuteraHeader),
    SuteraStatus(sutera_status::SuteraStatus),
    OneshotHeaders(oneshot_headers::OneshotHeader),
    Content(Vec<u8>),
    Unfragmented(Vec<u8>),
}

pub struct ClockingConnection {
    stream: TcpStream,
    buffer: BytesMut,
    author: MessageAuthor,
    context: ConnectionContext,
}
impl ClockingConnection {
    /// 既存のストリームから新しいClockingConnectionを作成します。
    ///
    /// **authorには、名前の通り「メッセージの送信者」が格納されることに注意してください。**
    /// たとえば、サーバー側で動いている場合は、[`MessageAuthor::Client`]を`author`に指定する必要があります。`
    pub fn new(stream: TcpStream, author: MessageAuthor) -> Self {
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

    async fn parse_frame(&mut self) -> Result<Option<ClockingFrameUnit>, ClockingFramingError> {
        let mut buf = Cursor::new(&self.buffer[..]);
        match self.context {
            ConnectionContext::None => {
                let remaining = buf.remaining();

                // SuteraHeader (通信のはじまりの目印) を探す
                // この時点で、SuteraHeaderの最小サイズよりも小さい場合は、次のバッファも読む
                if remaining < sutera_header::SuteraHeader::MINIMAL_FRAME_SIZE {
                    return Ok(None);
                }

                // 先頭からSuteraHeaderが成立していれば文句なしでOK
                buf.set_position(0);
                if let Some(header) =
                    sutera_header::SuteraHeader::parse_frame_unchecked(&mut buf, &()).await
                {
                    self.buffer.advance(buf.position() as usize);
                    self.context =
                        ConnectionContext::WaitStatus(header.message_length as usize - remaining);
                    return Ok(Some(ClockingFrameUnit::SuteraHeader(header)));
                }

                // 処理がここまで流れている時点で、
                // 最後にフレームが成立してから直ちにヘッダーが来ていないから何かがおかしい
                //
                // ただ、一応次にどこかでSuteraHeaderが来るかもしれないので、
                // バッファのどこかにSuteraHeaderを検知できたらそれまでのところをUnfragmentedとする
                for i in 1..=(remaining - sutera_header::SuteraHeader::MINIMAL_FRAME_SIZE) {
                    buf.set_position(i as u64);
                    if (sutera_header::SuteraHeader::parse_frame_unchecked(&mut buf, &()).await)
                        .is_some()
                    {
                        return Ok(Some(ClockingFrameUnit::Unfragmented(
                            self.buffer.copy_to_bytes(i).to_vec(),
                        )));
                    }
                }

                // 認識されていない状態でバッファが増えつづけると危険なので、
                // 定期的にUnfragmentedとして処理する
                if remaining > 1024 {
                    Ok(Some(ClockingFrameUnit::Unfragmented(
                        self.buffer.copy_to_bytes(1024).to_vec(),
                    )))
                } else {
                    Ok(None)
                }
            }
            ConnectionContext::WaitStatus(_) => todo!(),
            ConnectionContext::WaitMessageType(_) => todo!(),
            ConnectionContext::WaitContent(_) => todo!(),
        }
    }
}
