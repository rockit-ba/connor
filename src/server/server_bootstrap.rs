//! connor server_bootstrap

use crate::models::{NewService, RpcKind, TcpWriter};

use crate::custom_error::Byte2JsonErr;
use crate::server::{
    deregistry_handle, discovery_handle, discoverynames_handle, registry_handle,
    servicecheck_handle,
};
use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
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
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.addr.as_str()).await?;
        info!("Connor server_bootstrap startup");
        let mut listener_stream = TcpListenerStream::new(listener);

        while let Some(socket) = listener_stream.try_next().await? {
            let arc_map = self.servers.clone();
            info!("connection come in：{:?}", &socket.local_addr().unwrap());

            tokio::spawn(async move {
                let framed = Framed::new(socket, LengthDelimitedCodec::new());
                let (writer, reader) = &mut framed.split();
                // 注意这里的Ok(Some(req)) 不能拆开写，这样会导致一直 ok()
                while let Ok(Some(req)) = reader.try_next().await {
                    let string = String::from_utf8((&req).to_vec())
                        .unwrap_or_else(|_| panic!("{}", Byte2JsonErr));

                    info!("inbound data：{}", string);

                    if let Ok(rpc_kind) = RpcKind::from_str(&string[0..1]) {
                        let json = &string[1..];
                        inbound_handle(rpc_kind, json, writer, arc_map.clone()).await;
                    }
                }
                warn!("socket close \n");
            });
        }
        Ok(())
    }
}

/// 根据解析后的请求类型 和 json 体进行后续处理
// #[instrument]
async fn inbound_handle(rpc_kind: RpcKind, json: &str, writer: &mut TcpWriter, map: ServersMap) {
    match rpc_kind {
        // 服务注册
        RpcKind::Registry => {
            registry_handle::handle(json, writer, map).await;
        }
        // 服务发现：根据service-name 获取所有的service
        RpcKind::Discovery => {
            discovery_handle::handle(json, writer, map).await;
        }
        // 获取所有的service-names
        RpcKind::DiscoveryNames => {
            discoverynames_handle::handle(json, writer, map).await;
        }
        // 服务下线
        RpcKind::Deregistry => {
            deregistry_handle::handle(json, writer, map).await;
        }
        // 服务检测
        RpcKind::ServiceCheck => {
            servicecheck_handle::handle(json, writer, map).await;
        }
    }
}
