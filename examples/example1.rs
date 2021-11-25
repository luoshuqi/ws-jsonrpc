use std::convert::Infallible;
use std::env::set_var;
use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info};
use tokio::net::{TcpListener, TcpStream};
use ws::request::Request;
use ws::response::{Response, NOT_FOUND};
use ws::websocket::WebSocket;

use ws_jsonrpc::handler::Handler;
use ws_jsonrpc::{method, rpc};

#[rpc]
async fn greeting(name: String) -> Result<String, Infallible> {
    Ok(format!("Hello, {}", name))
}

#[rpc]
fn next_id() -> Result<u64, Infallible> {
    static ID: AtomicU64 = AtomicU64::new(1);
    Ok(ID.fetch_add(1, Ordering::Relaxed))
}

fn create_handler() -> Arc<Handler> {
    let mut handler = Handler::new();
    handler.register(vec![method!(greeting), method!(next_id)]);
    Arc::new(handler)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    info!("server started at {}", listener.local_addr()?);

    let handler = create_handler();

    loop {
        let (stream, addr) = listener.accept().await?;
        debug!("new connection from {}", addr);

        let handler = Arc::clone(&handler);
        tokio::spawn(async move {
            match handle_client(stream, handler).await {
                Ok(()) => {}
                Err(err) => error!("{} {:?}", addr, err),
            }
        });
    }
}

async fn handle_client(mut stream: TcpStream, handler: Arc<Handler>) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0u8; 1024];
    let req = Request::new(&mut stream, &mut buf, Duration::from_secs(60)).await?;
    match req.uri().split('?').next().unwrap() {
        "/" => match WebSocket::upgrade(&req, stream).await? {
            Some(ws) => handler.handle(ws).await?,
            None => {}
        },
        _ => Response::status(NOT_FOUND).write(&mut stream).await?,
    }
    Ok(())
}
