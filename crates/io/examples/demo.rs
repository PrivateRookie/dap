use clap::Parser;
use dap_io::AsyncCodec;
use dap_ty::{Capabilities, FromReq, InitializeRequestArguments, OneOf, OneOf3, Request, Response};
use tokio::{net::TcpStream, sync::Mutex};

pub use std::io::Result as IOResult;
use std::sync::Arc;

#[derive(Debug, Clone, Parser)]
struct Args {
    /// listening addr, if use ipv6, wrap addr by `[]`
    #[clap(long, default_value = "127.0.0.1")]
    pub host: String,
    /// listening port
    #[clap(long, short, default_value = "9595")]
    pub port: u16,
    /// enable debug level logging
    #[clap(long, short, default_value = "info")]
    pub level: tracing::Level,
}

async fn params_error(
    _server: Arc<Mutex<&mut Server>>,
    _seq: i64,
    e: serde_json::Error,
) -> IOResult<()> {
    // server
    // .lock()
    // .await
    // .codec
    // .send_resp(Response::err_resp(
    //     seq,
    //     ResponseError {
    //         code: -32602,
    //         data: None,
    //         message: e.to_string(),
    //     },
    // ))
    // .await
    tracing::error!("{:?}", e);
    Ok(())
}

pub struct Server {
    pub codec: AsyncCodec<TcpStream>,
}

impl Server {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            codec: AsyncCodec::new(stream),
        }
    }

    pub async fn receive(&mut self) -> IOResult<()> {
        match self.codec.receive().await? {
            OneOf3::This(req) => self.on_req(req).await,
            OneOf3::Among(_resp) => todo!(),
            OneOf3::Other(_event) => todo!(),
        }
    }

    pub async fn on_req(&mut self, req: Request) -> IOResult<()> {
        req.with(Arc::new(Mutex::new(self)), params_error)
            .async_then(|ctx, seq, _: InitializeRequestArguments| async move {
                let ctx = &mut ctx.lock().await;
                let body = Capabilities {
                    supports_configuration_done_request: Some(true),
                    ..Default::default()
                };
                ctx.codec
                    .send_resp(Response::ok_with(
                        seq,
                        InitializeRequestArguments::COMMAND,
                        body,
                    ))
                    .await
            })
            .await
            .async_unify(|req| async move {
                let (req, ctx, _) = req.split();
                tracing::warn!("unhandled {:#?}", req);
                let mut ctx = ctx.lock().await;
                OneOf::This(
                    ctx.codec
                        .send_resp(Response::ok_with(req.seq, &req.command, ()))
                        .await,
                )
            })
            .await
            .async_unify(|x| async move { x })
            .await
    }
}

#[tokio::main]
async fn main() {}
