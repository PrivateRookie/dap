use dap_ty::{Event, OneOf3, Request, Response};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::utils::CodecState;

type IOResult<T> = std::io::Result<T>;

/// async protocol message reader/writer
pub struct AsyncCodec<S: AsyncRead + AsyncWrite> {
    stream: S,
    state: CodecState,
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncCodec<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            state: CodecState::default(),
        }
    }

    /// get mutable ref of underlying stream
    pub fn stream_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    async fn poll(&mut self) -> tokio::io::Result<usize> {
        let state = &mut self.state;
        let count = self.stream.read(&mut state.read_buf).await?;
        if count == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "read eof",
            ));
        }
        state.read_data.extend_from_slice(&state.read_buf[..count]);
        Ok(count)
    }

    pub async fn receive(&mut self) -> tokio::io::Result<OneOf3<Request, Response, Event>> {
        loop {
            if let Some(may_ok) = self.state.try_parse_header() {
                may_ok.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                break;
            } else {
                self.poll().await?;
            }
        }

        while !self.state.body_ready() {
            self.poll().await?;
        }

        self.state
            .consume_body()
            .map_err(|e| tokio::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// write message to peer
    pub async fn send(
        &mut self,
        message: OneOf3<Request, Response, Event>,
    ) -> tokio::io::Result<()> {
        let json_str = serde_json::to_string(&message)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let data = json_str.as_bytes();
        self.stream
            .write_all(format!("Content-Length: {}\r\n\r\n", data.len(),).as_bytes())
            .await?;
        self.stream.write_all(data).await?;
        Ok(())
    }

    /// helper function to send request only
    pub async fn send_req(&mut self, message: Request) -> tokio::io::Result<()> {
        self.send(OneOf3::This(message)).await
    }

    /// helper function to send response only
    pub async fn send_resp(&mut self, message: Response) -> tokio::io::Result<()> {
        self.send(OneOf3::Among(message)).await
    }

    /// helper function to send notification only
    pub async fn send_event(&mut self, message: Event) -> tokio::io::Result<()> {
        self.send(OneOf3::Other(message)).await
    }
}

#[cfg(feature = "async_ws")]
mod ws_codec {

    use dap_ty::{Event, OneOf3, Request, Response};
    use tokio::net::TcpStream;
    use ws_tool::{
        codec::{default_handshake_handler, AsyncWsStringCodec},
        frame::OpCode,
        stream::WsAsyncStream,
        ClientBuilder, ServerBuilder,
    };

    use super::IOResult;

    pub struct AsyncWsCodec {
        ws: AsyncWsStringCodec<WsAsyncStream<TcpStream>>,
    }

    impl AsyncWsCodec {
        pub async fn new_client<S: ToString>(addr: S) -> IOResult<Self> {
            let ws = ClientBuilder::new(addr)
                .async_connect(AsyncWsStringCodec::check_fn)
                .await?;
            Ok(Self { ws })
        }

        pub async fn new_server(stream: TcpStream) -> IOResult<Self> {
            let ws = ServerBuilder::async_accept(
                stream,
                default_handshake_handler,
                AsyncWsStringCodec::factory,
            )
            .await?;
            Ok(Self { ws })
        }

        pub async fn receive(&mut self) -> IOResult<OneOf3<Request, Response, Event>> {
            let msg = self.ws.receive().await?;
            if msg.code == OpCode::Close {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "peer send close",
                ));
            } else if msg.code == OpCode::Text {
                let msg = serde_json::from_str(&msg.data).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                })?;
                Ok(msg)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unknown frame code {:?}", msg.code),
                ))
            }
        }

        pub async fn close(&mut self, status: u16, msg: String) -> IOResult<()> {
            self.ws.send((status, msg)).await?;
            Ok(())
        }

        pub async fn send(&mut self, message: OneOf3<Request, Response, Event>) -> IOResult<()> {
            let json_str = serde_json::to_string(&message)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            self.ws.send(json_str).await?;
            Ok(())
        }

        /// helper function to send request only
        pub async fn send_req(&mut self, message: Request) -> IOResult<()> {
            self.send(OneOf3::This(message)).await
        }

        /// helper function to send response only
        pub async fn send_resp(&mut self, message: Response) -> IOResult<()> {
            self.send(OneOf3::Among(message)).await
        }

        /// helper function to send notification only
        pub async fn send_event(&mut self, message: Event) -> IOResult<()> {
            self.send(OneOf3::Other(message)).await
        }
    }
}

#[cfg(feature = "async_ws")]
pub use ws_codec::AsyncWsCodec;
