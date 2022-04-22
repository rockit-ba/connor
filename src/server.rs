//! connor server

use crate::models::{DeregistryRequest, DiscoveryRequest, DiscoveryResponse, DiscoveryServiceNamesRequest, DiscoveryServiceNamesResponse, NewService, RegistryRequest, RegistryResponse, RpcCodec, RpcKind, ServiceCheckRequest, ServiceCheckResponse, TcpWriter};

use anyhow::Result;
use bytes::Bytes;
use futures::{SinkExt, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::{Arc};
use parking_lot::RwLock;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, info, warn};
use crate::custom_error::{Byte2JsonErr};

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
        info!("Connor server startup");
        let mut listener_stream = TcpListenerStream::new(listener);

        while let Some(socket) = listener_stream.try_next().await? {
            let arc_map = self.servers.clone();
            info!("有连接进入：{:?}", &socket.local_addr().unwrap());

            tokio::spawn(async move {
                let framed = Framed::new(socket, LengthDelimitedCodec::new());
                let (writer, reader) = &mut framed.split();
                // 注意这里的Ok(Some(req)) 不能拆开写，这样会导致一直 ok()
                while let Ok(Some(req)) = reader.try_next().await {
                    let string = String::from_utf8((&req).to_vec())
                        .unwrap_or_else(|_| { panic!("{}", Byte2JsonErr) });

                    info!("入参：{}", string);

                    if let Ok(rpc_kind) = RpcKind::from_str(&string[0..1]) {
                        let json = &string[1..];
                        inbound_handle(rpc_kind, json, writer, arc_map.clone()).await;
                    }
                }
                warn!("socket 已回收 \n");
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
            let registry_req = RegistryRequest::from_json(json);
            info!("解码入站数据 {:?}", &registry_req);
            // 存储注册的服务
            let service = &registry_req.service;
            {
                let mut servers = map.write();
                match servers.get_mut(&service.name) {
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
            let content = registry_response.to_json();
            if let Err(err) = writer
                .send(Bytes::copy_from_slice(content.as_bytes()))
                .await
            {
                error!("{:?}", err);
            }
        }
        // 服务发现：根据service-name 获取所有的service
        RpcKind::Discovery => {
            let discovery_req = DiscoveryRequest::from_json(json);
            info!("解码入站数据 {:?}", &discovery_req);
            let mut services = None;
            {
                let map = map.read();
                if let Some(lists) = map.get(&discovery_req.service_name) {
                    services = Some(lists.clone());
                }
            }
            let discovery_response = DiscoveryResponse::new(&discovery_req.service_name, services);
            let content = discovery_response.to_json();

            info!("回送service list：{}",&content);
            if let Err(err) = writer
                .send(Bytes::copy_from_slice(content.as_bytes()))
                .await
            {
                error!("{:?}", err);
            }
        }
        // 获取所有的service-names
        RpcKind::DiscoveryNames => {
            let service_names_request = DiscoveryServiceNamesRequest::from_json(json);
            info!("解码入站数据 {:?}", &service_names_request);
            let service_names;
            {
                let map = map.read();
                service_names = map.keys().cloned()
                    .collect();
            }
            let names_response = DiscoveryServiceNamesResponse::new(service_names);
            let content = names_response.to_json();

            info!("回送service names：{}",&content);
            if let Err(err) = writer
                .send(Bytes::copy_from_slice(content.as_bytes()))
                .await
            {
                error!("{:?}", err);
            }

        }
        // 服务下线
        RpcKind::Deregistry => {
            let deregistry_request = DeregistryRequest::from_json(json);
            info!("解码入站数据 {:?}", &deregistry_request);
            {
                let mut map = map.write();
                if let Some(services) = map.get_mut(&deregistry_request.service_name) {
                    *services = services.iter()
                        .filter(|&service| {
                            service.id.ne(&deregistry_request.service_id)
                        }).cloned()
                        .collect();
                }
            }

        }
        // f服务检测
        RpcKind::ServiceCheck => {
            let check_request = ServiceCheckRequest::from_json(json);
            info!("解码入站数据 {:?}", &check_request);
            let service_id:String;
            {
                let map = map.read();
                service_id = map.values()
                    .flat_map(|ele| {
                        ele.iter()
                            .map(|ele| ele.id.clone())
                            .filter(|ele| {ele.eq(&check_request.service_id)})
                    }).collect();
            }
            info!("{}",&service_id);
            let response = ServiceCheckResponse { service_id };
            let content = response.to_json();
            info!("回送service-id：{}",&content);
            if let Err(err) = writer
                .send(Bytes::copy_from_slice(content.as_bytes()))
                .await
            {
                error!("{:?}", err);
            }
        }
    }
}
