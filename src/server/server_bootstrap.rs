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
use std::time::{SystemTime};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::time::{sleep};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, info, warn};

/// 存放已经注册进来的所有的服务，key是service-name
pub type ServersMap = Arc<RwLock<HashMap<String, Vec<NewService>>>>;
/// 存放心跳请求数据（<实例ID, timestamp>）
pub type ServersHeartbeatMap = Arc<RwLock<HashMap<String, SystemTime>>>;

/// Connor 服务
pub struct ConnorServer {
    // 启动地址
    addr: String,
    // 注册的服务
    servers: ServersMap,
    // 心跳请求数据
    servers_heartbeat: ServersHeartbeatMap,
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
            servers_heartbeat: ServersHeartbeatMap::new(RwLock::new(HashMap::<String, SystemTime>::new())),
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

        let services_heartbeat_map = self.servers_heartbeat.clone();
        let services_map = self.servers.clone();
        let heartbeat_publisher = broad_tx.clone();
        // 定时检测心跳数据
        tokio::spawn(async move {
            // 每90 秒进行检测
            sleep(tokio::time::Duration::from_secs(90)).await;
            // 超时 ID 集合，这些 instance_id都要从servers_map中移除
            let timeout_instance_ids;
            {
                let read_guard = services_heartbeat_map.read();
                // 获取超时的instance_id(当前时间差超过 90秒即为过期)
                timeout_instance_ids = read_guard.iter()
                    .filter(|(_, system_time)| {
                        if let Ok(time) = system_time.elapsed() {
                            return time.as_secs() > 90;
                        }
                        false
                    }).map(|(id,_)| { id.clone() })
                    .collect::<Vec<String>>();
            }
            {
                let mut write_guard = services_map.write();
                // 移除超时的instance_id
                let health_services_map = write_guard.iter().map(|(service_name, services)| {
                    let health_services = services.iter().filter(|service| {
                        !timeout_instance_ids.contains(&service.id)
                    }).map(|service| {
                        service.clone()
                    }).collect();
                    (service_name.to_string(), health_services)
                }).collect::<HashMap<String, Vec<NewService>>>();

                *write_guard = health_services_map;
            }
            // 将timeout_instance_ids进行广播，客户端需要移除
            if let Err(err) = heartbeat_publisher.send(
                InboundHandleBroadcastEvent::HeartbeatTimeoutResp { service_ids: timeout_instance_ids })
            {
                error!("heartbeat_publisher send error: {}", err);
            }
        });

        while let Some(socket) = listener_stream.try_next().await? {
            let peer_addr = socket.peer_addr().unwrap().to_string();
            info!("connection come in：{}", &peer_addr);

            let (m_sender, mut s_receiver) = mpsc::channel::<InboundHandleSingleEvent>(16);

            let services_map = self.servers.clone();
            let services_heartbeat_map = self.servers_heartbeat.clone();

            // channel
            let (writer, mut reader) = Framed::new(socket, LengthDelimitedCodec::new()).split();
            let writer = Arc::new(Mutex::new(writer));

            // response client spawn
            // 用于监听处理响应客户端的请求(单消费者响应)
            let single_writer = writer.clone();
            let single_handle = tokio::spawn(async move {
                while let Some(data) = s_receiver.recv().await {
                    outbound_handle(data, single_writer.clone()).await;
                }
            });

            // 多消费者响应
            let mut broad_receiver = broad_tx.subscribe();
            let broad_writer = writer.clone();
            let broad_handle = tokio::spawn(async move {
                while let Ok(data) = broad_receiver.recv().await {
                    outbound_broad_handle(data, broad_writer.clone()).await;
                }
            });

            // 请求处理
            // 用来发送响应客户端的消息
            let broad_sender = broad_tx.clone();
            tokio::spawn(async move {
                while let Ok(Some(req)) = reader.try_next().await {
                    let string = String::from_utf8((&req).to_vec())
                        .unwrap_or_else(|_| panic!("{}", Byte2JsonErr));
                    info!("Inbound data：{}", string);

                    if let Ok(rpc_kind) = RpcKind::from_str(&string[0..1]) {
                        let json = &string[1..];
                        inbound_handle(rpc_kind, json,
                                       &broad_sender,
                                       &m_sender,
                                       services_map.clone(),
                                       services_heartbeat_map.clone()
                        ).await;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut map: HashMap<i32, i32> = (0..8).map(|x| (x, x*10)).collect();
        map.retain(|&k, _| k % 2 == 0);
        println!("{:?}", map);
    }
}
