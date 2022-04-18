//! connor server

use crate::common::{DiscoveryRequest, NewService, RegistryRequest, RegistryResponse, TcpWriter};
use crate::common::{DISCOVERY, REGISTRY};

use anyhow::Result;
use bytes::Bytes;
use futures::{SinkExt, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

/// 存放已经注册进来的所有的服务，key是service-name
type ServersMap = Arc<Mutex<HashMap<String, Vec<NewService>>>>;

/// Connor 服务
pub struct ConnorServer {
    // 启动地址
    addr: String,
    // 注册的服务
    servers: ServersMap,
}
impl Default for ConnorServer {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8080".to_string(),
            servers: ServersMap::new(Mutex::new(HashMap::<String, Vec<NewService>>::new())),
        }
    }
}
impl ConnorServer {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.addr.as_str()).await?;
        println!("Connor server startup");
        let mut listener_stream = TcpListenerStream::new(listener);

        while let Some(socket) = listener_stream.try_next().await? {
            let arc_map = self.servers.clone();
            println!("有连接进入：{:?}", &socket.local_addr().unwrap());
            // 每次有一个客户端的连接进来就创建一个 任务
            // 如果不使用 move 是不能使用swap 外部的变量的，用了move之后，该数据就只能被当前的 任务使用。
            // 当然可以使用 Arc
            tokio::spawn(async move {
                let framed = Framed::new(socket, LengthDelimitedCodec::new());
                let (writer, reader) = &mut framed.split();
                // 注意这里的Ok(Some(req)) 不能拆开写，这样会导致一直 ok()
                while let Ok(Some(req)) = reader.try_next().await {
                    let string = String::from_utf8((&req).to_vec()).expect("byte to json fail");
                    println!("{}", string);
                    let rpc_kind = &string[0..1];
                    let json = &string[1..];
                    inbound_handle(rpc_kind, json, writer, arc_map.clone()).await;
                }
                println!("socket 已回收");
            });
        }
        Ok(())
    }
}

/// 根据解析后的请求类型 和 json 体进行后续处理
async fn inbound_handle(rpc_kind: &str, json: &str, writer: &mut TcpWriter, map: ServersMap) {
    match rpc_kind {
        // 服务注册
        REGISTRY => {
            let registry_req =
                serde_json::from_str::<RegistryRequest>(json).expect("json to struct fail");
            println!("解码入站数据 {:?}", &registry_req);
            // 存储注册的服务
            let service = &registry_req.service;
            {
                let mut servers = map.lock().unwrap();
                match servers.get_mut(service.name.as_str()) {
                    Some(list) => {
                        list.push(service.clone());
                    }
                    None => {
                        servers.insert(service.name.clone(), vec![service.clone()]);
                    }
                }
            }
            // 响应注册服务
            let registry_response = RegistryResponse::new(true, &registry_req.service.name);
            let json = serde_json::to_string(&registry_response).expect("struct to json fail");
            let content = format!("{}{}", REGISTRY, json);
            if let Err(err) = writer
                .send(Bytes::copy_from_slice(content.as_bytes()))
                .await
            {
                println!("{:?}", err);
            }
        }
        // 服务发现：根据service-name 获取所有的service
        DISCOVERY => {
            let discovery_req =
                serde_json::from_str::<DiscoveryRequest>(json).expect("json to struct fail");
            println!("解码入站数据 {:?}", &discovery_req);
        }
        _ => {}
    }
}
