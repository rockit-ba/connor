//! 消息入站处理模块

mod deregistry;
mod discovery;
mod discovery_names;
mod registry;
mod service_check;

use crate::models::InboundHandleSingleEvent::ServiceDeregistryResp;
use crate::models::{InboundHandleBroadcastEvent, InboundHandleSingleEvent, RpcKind};
use crate::server_bootstrap::ServersMap;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::Sender as SingleSender;
use tracing::error;

/// 根据解析后的请求类型 和 json 体进行后续处理
// #[instrument]
pub async fn inbound_handle(
    rpc_kind: RpcKind,
    json: &str,
    broad: &Sender<InboundHandleBroadcastEvent>,
    sender: &SingleSender<InboundHandleSingleEvent>,
    map: ServersMap,
) {
    match rpc_kind {
        // 服务注册
        RpcKind::Registry => {
            let new_service = registry::handle(json, map).await;
            // 首先发布此次请求的响应事件
            response(
                sender,
                InboundHandleSingleEvent::ServiceRegistryResp { success: true },
            )
            .await;
            // 然后发布更新客户端缓存信息的事件，由Connor 主动向 client 发送服务刷新请求
            publisher(broad, new_service);
        }
        // 服务发现：根据service-name 获取所有的service
        RpcKind::Discovery => {
            let handle_event = discovery::handle(json, map).await;
            response(sender, handle_event).await;
        }
        // 获取所有的service-names
        RpcKind::DiscoveryNames => {
            let handle_event = discovery_names::handle(json, map).await;
            response(sender, handle_event).await;
        }
        // 服务下线
        RpcKind::Deregistry => {
            let deregistry_request = deregistry::handle(json, map).await;
            // 同样的这里首先也需要发送响应此次客户端的事件
            response(sender, ServiceDeregistryResp { success: true }).await;
            // 然后需要主动通知客户端更新缓存（删除这个服务）
            publisher(
                broad,
                InboundHandleBroadcastEvent::RemoveServiceResp {
                    service_id: deregistry_request.service_id,
                    service_name: deregistry_request.service_name,
                },
            )
        }
        // 服务检测
        RpcKind::ServiceCheck => {
            let handle_event = service_check::handle(json, map).await;
            response(sender, handle_event).await;
        }
        RpcKind::AddService => {}
        RpcKind::RemoveService => {}
    }
}
// 发布事件消息
async fn response(
    sender: &SingleSender<InboundHandleSingleEvent>,
    handle_event: InboundHandleSingleEvent,
) {
    if let Err(result) = sender.send(handle_event).await {
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
