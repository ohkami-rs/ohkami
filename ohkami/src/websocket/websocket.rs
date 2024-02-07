use std::io::Error;
use super::{Message};
use crate::__rt__::{AsyncWriter, AsyncReader, TcpStream};

//#[cfg(test)]      use crate::layer6_testing::TestStream as Stream;
//#[cfg(not(test))] use crate::__rt__::TcpStream          as Stream;


/// In current version, `split` to read / write halves is not supported
pub struct WebSocket<Stream: AsyncReader + AsyncWriter = TcpStream> {
    stream: Stream,
    config: Config,

    n_buffered: usize,
}

// :fields may set through `WebSocketContext`'s methods
pub struct Config {
    pub(crate) write_buffer_size:      usize,
    pub(crate) max_write_buffer_size:  usize,
    pub(crate) max_message_size:       Option<usize>,
    pub(crate) max_frame_size:         Option<usize>,
    pub(crate) accept_unmasked_frames: bool,
} const _: () = {
    impl Default for Config {
        fn default() -> Self {
            Self {
                write_buffer_size:      128 * 1024, // 128 KiB
                max_write_buffer_size:  usize::MAX,
                max_message_size:       Some(64 << 20),
                max_frame_size:         Some(16 << 20),
                accept_unmasked_frames: false,
            }
        }
    }
};

impl<Stream: AsyncReader + AsyncWriter> WebSocket<Stream> {
    pub(crate) fn new(stream: Stream, config: Config) -> Self {
        Self { stream, config, n_buffered:0 }
    }
}

impl WebSocket {
    pub async fn recv(&mut self) -> Result<Option<Message>, Error> {
        Message::read_from(&mut self.stream, &self.config).await
    }
}

// =============================================================================
pub(super) async fn send(
    message:    Message,
    stream:     &mut (impl AsyncWriter + Unpin),
    config:     &Config,
    n_buffered: &mut usize,
) -> Result<(), Error> {
    message.write(stream, config).await?;
    flush(stream, n_buffered).await?;
    Ok(())
}
pub(super) async fn write(
    message:    Message,
    stream:     &mut (impl AsyncWriter + Unpin),
    config:     &Config,
    n_buffered: &mut usize,
) -> Result<usize, Error> {
    let n = message.write(stream, config).await?;

    *n_buffered += n;
    if *n_buffered > config.write_buffer_size {
        if *n_buffered > config.max_write_buffer_size {
            panic!("Buffered messages is larger than `max_write_buffer_size`");
        } else {
            flush(stream, n_buffered).await?
        }
    }

    Ok(n)
}
pub(super) async fn flush(
    stream:     &mut (impl AsyncWriter + Unpin),
    n_buffered: &mut usize,
) -> Result<(), Error> {
    stream.flush().await
        .map(|_| *n_buffered = 0)
}
// =============================================================================

impl WebSocket {
    pub async fn send(&mut self, message: Message) -> Result<(), Error> {
        send(message, &mut self.stream, &self.config, &mut self.n_buffered).await
    }
    pub async fn write(&mut self, message: Message) -> Result<usize, Error> {
        write(message, &mut self.stream, &self.config, &mut self.n_buffered).await
    }
    pub async fn flush(&mut self) -> Result<(), Error> {
        flush(&mut self.stream, &mut self.n_buffered).await
    }
}
