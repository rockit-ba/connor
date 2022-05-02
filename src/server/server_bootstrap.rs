//! connor server_bootstrap

use crate::models::{InboundHandleBroadcastEvent, InboundHandleSingleEvent, NewService, RpcKind};
use crate::custom_error::Byte2JsonErr;
use crate::server::outbound::outbound_broad_handle;
use crate::server::{inbound_handle, outbound_handle};
use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{info, warn};

/// 存放已经注册进来的所有的服务，key是service-name
pub type ServersMap = Arc<RwLock<HashMap<String, Vec<NewService>>>>;

/// Connor 服务
pub struct ConnorServer {
    // 启动地址
    addr: String,
    // 注册的服务
    servers: ServersMap,
}

impl Debug for ConnorServer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnorServer").finish()
    }
}
impl Default for ConnorServer {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8080".to_string(),
            servers: ServersMap::new(RwLock::new(HashMap::<String, Vec<NewService>>::new())),
        }
    }
}
impl ConnorServer {
    pub fn new() -> Self {
        Self::default()
    }

    // #[instrument]
    pub async fn start(&mut self) -> Result<()> {
        let listener = TcpListener::bind(self.addr.as_str()).await?;
        info!("Connor Server_Bootstrap Startup");
        let mut listener_stream = TcpListenerStream::new(listener);

        let (broad_tx, _) = broadcast::channel::<InboundHandleBroadcastEvent>(1024);
        while let Some(socket) = listener_stream.try_next().await? {
            let peer_addr = socket.peer_addr().unwrap().to_string();
            info!("connection come in：{}", &peer_addr);
            // client注册的服务的容器
            let (m_sender, mut s_receiver) = mpsc::channel::<InboundHandleSingleEvent>(16);
            let arc_map = self.servers.clone();
            // channel
            let (writer, mut reader) = Framed::new(socket, LengthDelimitedCodec::new()).split();
            let writer = Arc::new(Mutex::new(writer));
            // response client spawn
            // 用于监听处理响应客户端的请求
            let single_writer = writer.clone();
            let single_handle = tokio::spawn(async move {
                while let Some(data) = s_receiver.recv().await {
                    outbound_handle(data, single_writer.clone()).await;
                }
            });
            let mut broad_receiver = broad_tx.subscribe();
            let broad_writer = writer.clone();
            let broad_handle = tokio::spawn(async move {
                while let Ok(data) = broad_receiver.recv().await {
                    outbound_broad_handle(data, broad_writer.clone()).await;
                }
            });

            // 用来发送响应客户端的消息
            let broad_sender = broad_tx.clone();
            tokio::spawn(async move {
                while let Ok(Some(req)) = reader.try_next().await {
                    let string = String::from_utf8((&req).to_vec())
                        .unwrap_or_else(|_| panic!("{}", Byte2JsonErr));
                    info!("Inbound data：{}", string);

                    if let Ok(rpc_kind) = RpcKind::from_str(&string[0..1]) {
                        let json = &string[1..];
                        inbound_handle(rpc_kind, json, &broad_sender, &m_sender, arc_map.clone())
                            .await;
                    }
                }

                warn!("Reader Close\n");
                single_handle.abort();
                broad_handle.abort();
                warn!("Writer Close\n");
            });
        }
        Ok(())
    }
}
