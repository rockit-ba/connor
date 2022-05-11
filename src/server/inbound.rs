//! 消息入站处理模块

mod deregistry;
mod discovery;
mod discovery_names;
mod registry;
mod service_check;
mod heartbeat;

use crate::models::InboundHandleSingleEvent::ServiceDeregistryResp;
use crate::models::{InboundHandleBroadcastEvent, InboundHandleSingleEvent, RpcKind};
use crate::server_bootstrap::{ServersHeartbeatMap, ServersMap};
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::Sender as SingleSender;
use tracing::error;

pub struct InboundParams {
    rpc_kind: RpcKind,
    json: String,
    broad: Sender<InboundHandleBroadcastEvent>,
    unicast: SingleSender<InboundHandleSingleEvent>,
    services_map: ServersMap,
    services_heartbeat_map: ServersHeartbeatMap,
}

impl InboundParams {
    pub fn new(
        rpc_kind: RpcKind,
        json: String,
        broad: Sender<InboundHandleBroadcastEvent>,
        unicast: SingleSender<InboundHandleSingleEvent>,
        services_map: ServersMap,
        services_heartbeat_map: ServersHeartbeatMap,
    ) -> Self {
        Self {
            rpc_kind,
            json,
            broad,
            unicast,
            services_map,
            services_heartbeat_map,
        }
    }
}


/// 根据解析后的请求类型 和 json 体进行后续处理
// #[instrument]
pub async fn inbound_handle(params: InboundParams) {
    match params.rpc_kind {
        // 服务注册
        RpcKind::Registry => {
            let new_service = registry::handle(&params.json, params.services_map).await;
            // 首先发布此次请求的响应事件
            unicast(
                &params.unicast,
                InboundHandleSingleEvent::ServiceRegistryResp { success: true },
            )
            .await;
            // 然后发布更新客户端缓存信息的事件，由Connor 主动向 client 发送服务刷新请求
            publisher(&params.broad, new_service);
        }
        // 服务发现：根据service-name 获取所有的service
        RpcKind::Discovery => {
            let handle_event = discovery::handle(&params.json, params.services_map).await;
            unicast(&params.unicast, handle_event).await;
        }
        // 获取所有的service-names
        RpcKind::DiscoveryNames => {
            let handle_event = discovery_names::handle(&params.json, params.services_map).await;
            unicast(&params.unicast, handle_event).await;
        }
        // 服务下线
        RpcKind::Deregistry => {
            let deregistry_request = deregistry::handle(&params.json, params.services_map).await;
            // 同样的这里首先也需要发送响应此次客户端的事件
            unicast(&params.unicast, ServiceDeregistryResp { success: true }).await;
            // 然后需要主动通知客户端更新缓存（删除这个服务）
            publisher(
                &params.broad,
                deregistry_request,
            )
        }
        // 服务检测
        RpcKind::ServiceCheck => {
            let handle_event = service_check::handle(&params.json, params.services_map).await;
            unicast(&params.unicast, handle_event).await;
        }
        // 心跳检测请求
        RpcKind::Heartbeat => {
            heartbeat::handle(&params.json, params.services_heartbeat_map).await;
            unicast(&params.unicast, InboundHandleSingleEvent::HeartbeatResp {success: true}).await;
        }
        // 其他情况,都是server端主动推送的请求
        RpcKind::HeartbeatTimeout => {}
        RpcKind::AddService => {}
        RpcKind::RemoveService => {}
    }
}
// 单播发布事件消息
async fn unicast(
    unicast: &SingleSender<InboundHandleSingleEvent>,
    handle_event: InboundHandleSingleEvent,
) {
    if let Err(result) = unicast.send(handle_event).await {
        error!("Response Event Error [{:?}]", result);
    }
}
// 发布事件消息
fn publisher(
    sender: &Sender<InboundHandleBroadcastEvent>,
    handle_event: InboundHandleBroadcastEvent,
) {
    if let Err(result) = sender.send(handle_event) {
        error!("Publisher Event Error [{:?}]", result);
    }
}
