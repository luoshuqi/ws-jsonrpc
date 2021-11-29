use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, error};
use serde_json::to_vec;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use ws::websocket::{Message, Opcode, WebSocket};

use crate::request::Request;
use crate::response::Response;
use crate::{Error, Method};

#[derive(Default)]
pub struct Handler {
    methods: HashMap<&'static str, Method>,
}

impl Handler {
    pub fn new() -> Self {
        Handler {
            methods: HashMap::new(),
        }
    }

    pub fn register(&mut self, methods: Vec<(&'static str, Method)>) {
        for (name, method) in methods {
            self.methods.insert(name, method);
        }
    }

    pub async fn handle<T: AsyncRead + AsyncWrite + Unpin>(
        self: &Arc<Self>,
        mut ws: WebSocket<T>,
    ) -> Result<(), ws::error::Error> {
        let mut close_send = false;
        let (tx, mut rx) = unbounded_channel::<Response>();
        loop {
            tokio::select! {
                msg = ws.receive() => {
                    match msg? {
                        Some(msg) => match msg.opcode() {
                            Opcode::Text | Opcode::Binary => self.handle_request(msg.payload(), &tx),
                            Opcode::Ping => {
                                let payload = msg.payload().to_vec();
                                ws.send(Message::new(Opcode::Pong, &payload)).await?;
                            }
                            Opcode::Pong => {}
                            Opcode::Close if close_send => break,
                            Opcode::Close => {
                                close_send = true;
                                ws.send(Message::close(&[])).await?;
                            }
                        },
                        None => break,
                    }
                }
                resp = rx.recv() => {
                    match resp {
                        Some(resp) => match to_vec(&resp) {
                            Ok(data) => ws.send(Message::text(&data)).await?,
                            Err(err) => error!("serialize {:?} error: {:?}", resp, err),
                        },
                        None => unreachable!(),
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_request(self: &Arc<Self>, payload: &[u8], tx: &UnboundedSender<Response>) {
        match Request::parse(payload) {
            Ok(req) => {
                debug!("{}", req);
                let tx = tx.clone();
                let this = Arc::clone(self);
                tokio::spawn(async move {
                    if let Some(resp) = this.call(req).await {
                        let _ = tx.send(resp);
                    }
                });
            }
            Err(Some(resp)) => {
                let _ = tx.send(resp);
            }
            Err(None) => {}
        }
    }

    async fn call(self: &Arc<Self>, req: Request) -> Option<Response> {
        match self.methods.get(req.method.as_str()) {
            Some(method) => {
                let args = req.params.unwrap_or(Vec::new());
                match method(args).await {
                    Ok(result) if req.id.is_some() => Some(Response::ok(result, req.id.unwrap())),
                    Err(err) if req.id.is_some() => Some(Response::error(err, req.id)),
                    _ => None,
                }
            }
            None if req.id.is_some() => Some(Response::error(
                Error::method_not_found(Some(format!("{}", req.method))),
                req.id,
            )),
            _ => None,
        }
    }
}
