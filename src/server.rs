//! connor server

use crate::common::{DiscoveryRequest, DiscoveryResponse, NewService, RegistryRequest, RegistryResponse, TcpWriter};
use crate::common::{DISCOVERY, REGISTRY};

use anyhow::Result;
use bytes::Bytes;
use futures::{SinkExt, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc};
use parking_lot::RwLock;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, info, warn};

/// 存放已经注册进来的所有的服务，key是service-name
type ServersMap = Arc<RwLock<HashMap<String, Vec<NewService>>>>;

/// Connor 服务
pub struct ConnorServer {
    // 启动地址
    addr: String,
    // 注册的服务
    servers: ServersMap,
}

impl Debug for ConnorServer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnorServer")
            .finish()
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
        info!("Connor server startup");
        let mut listener_stream = TcpListenerStream::new(listener);

        while let Some(socket) = listener_stream.try_next().await? {
            let arc_map = self.servers.clone();
            info!("有连接进入：{:?}", &socket.local_addr().unwrap());
            // 每次有一个客户端的连接进来就创建一个 任务
            // 如果不使用 move 是不能使用swap 外部的变量的，用了move之后，该数据就只能被当前的 任务使用。
            // 当然可以使用 Arc
            tokio::spawn(async move {
                let framed = Framed::new(socket, LengthDelimitedCodec::new());
                let (writer, reader) = &mut framed.split();
                // 注意这里的Ok(Some(req)) 不能拆开写，这样会导致一直 ok()
                while let Ok(Some(req)) = reader.try_next().await {
                    let string = String::from_utf8((&req).to_vec()).expect("byte to json fail");
                    info!("入参：{}", string);
                    let rpc_kind = &string[0..1];
                    let json = &string[1..];
                    inbound_handle(rpc_kind, json, writer, arc_map.clone()).await;
                }
                warn!("socket 已回收 \n");
            });
        }
        Ok(())
    }
}

/// 根据解析后的请求类型 和 json 体进行后续处理
// #[instrument]
async fn inbound_handle(rpc_kind: &str, json: &str, writer: &mut TcpWriter, map: ServersMap) {
    match rpc_kind {
        // 服务注册
        REGISTRY => {
            let registry_req =
                serde_json::from_str::<RegistryRequest>(json).expect("json to struct fail");
            info!("解码入站数据 {:?}", &registry_req);
            // 存储注册的服务
            let service = &registry_req.service;
            {
                let mut servers = map.write();
                match servers.get_mut(service.name.as_str()) {
                    Some(list) => {
                        list.push(service.clone());
                    }
                    None => {
                        servers.insert(service.name.clone(), vec![service.clone()]);
                    }
                }
                info!("当前服务列表：{:?}",&servers);
            }

            // 响应注册服务
            let registry_response = RegistryResponse::new(true, &registry_req.service.name);
            let json = serde_json::to_string(&registry_response).expect("struct to json fail");
            let content = format!("{}{}", REGISTRY, json);
            if let Err(err) = writer
                .send(Bytes::copy_from_slice(content.as_bytes()))
                .await
            {
                error!("{:?}", err);
            }
        }
        // 服务发现：根据service-name 获取所有的service
        DISCOVERY => {
            let discovery_req =
                serde_json::from_str::<DiscoveryRequest>(json).expect("json to struct fail");
            info!("解码入站数据 {:?}", &discovery_req);
            let mut services = Vec::<NewService>::new();

            {
                let map = map.read();
                if let Some(lists) = map.get(&discovery_req.service_name) {
                    services.append(&mut lists.clone());
                }
            }
            let discovery_response = DiscoveryResponse::new(&discovery_req.service_name, services);
            let json = serde_json::to_string(&discovery_response).expect("struct to json fail");
            let content = format!("{}{}", DISCOVERY, json);

            info!("回送service list：{}",&content);
            if let Err(err) = writer
                .send(Bytes::copy_from_slice(content.as_bytes()))
                .await
            {
                error!("{:?}", err);
            }
        }
        _ => {}
    }
}
