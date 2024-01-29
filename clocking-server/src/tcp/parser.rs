use suteravr_lib::clocking::{traits::MessageAuthor, ClockingConnection};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct ClientMessageStream<W: AsyncReadExt + AsyncWriteExt + Unpin> {
    connection: ClockingConnection<W>,
}

impl<W: AsyncReadExt + AsyncWriteExt + Unpin> ClientMessageStream<W> {
    pub fn new(stream: W) -> Self {
        let connection = ClockingConnection::new(stream, MessageAuthor::Client);
        Self { connection }
    }
}
