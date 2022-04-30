//! connor server_bootstrap

use crate::models::{InboundHandleBroadcastEvent, InboundHandleSingleEvent, NewService, RpcKind};

use crate::custom_error::Byte2JsonErr;
use crate::server::{inbound_handle, outbound_handle};
use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast,mpsc};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{info, warn};
use crate::server::outbound::outbound_broad_handle;

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

        let (tx, _) = broadcast::channel::<InboundHandleBroadcastEvent>(16);

        while let Some(socket) = listener_stream.try_next().await? {
            let peer_addr = socket.peer_addr().unwrap().to_string();
            info!("connection come in：{}", &peer_addr);
            // client注册的服务的容器
            let (m_sender, mut s_receiver) = mpsc::channel::<InboundHandleSingleEvent>(1);
            let arc_map = self.servers.clone();
            // channel
            let (mut writer, mut reader) =
                Framed::new(socket, LengthDelimitedCodec::new()).split();
            // response client spawn
            // 用于监听处理响应客户端的请求
            let mut receiver = tx.subscribe();
            let response_handle = tokio::spawn(async move {
                loop {
                    if let Ok(data) = receiver.recv().await {
                        outbound_broad_handle(data,&mut writer).await;
                    }
                    if let Some(data) = s_receiver.recv().await{
                        outbound_handle(data,&mut writer).await;
                    };
                }
            });

            // 用来发送响应客户端的消息
            let mut sender = tx.clone();
            tokio::spawn(async move {
                let m_sender = m_sender;
                // 注意这里的Ok(Some(req)) 不能拆开写，这样会导致一直 ok()
                while let Ok(Some(req)) = reader.try_next().await {
                    let string = String::from_utf8((&req).to_vec())
                        .unwrap_or_else(|_| panic!("{}", Byte2JsonErr));
                    info!("Inbound data：{}", string);

                    if let Ok(rpc_kind) = RpcKind::from_str(&string[0..1]) {
                        let json = &string[1..];
                        inbound_handle(rpc_kind, json, &mut sender, &m_sender,arc_map.clone()).await;
                    }
                }

                warn!("Reader Close\n");
                response_handle.abort();
                warn!("Writer Close\n");
            });

        }
        Ok(())
    }
}
