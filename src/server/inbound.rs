//! 消息入站处理模块

mod deregistry;
mod discovery;
mod discovery_names;
mod registry;
mod service_check;

use crate::models::{InboundHandleEvent, RpcKind};
use crate::server_bootstrap::ServersMap;
use tokio::sync::broadcast::Sender;
use tracing::error;

/// 根据解析后的请求类型 和 json 体进行后续处理
// #[instrument]
pub async fn inbound_handle(
    rpc_kind: RpcKind,
    json: &str,
    sender: &mut Sender<InboundHandleEvent>,
    map: ServersMap,
) {
    match rpc_kind {
        // 服务注册
        RpcKind::Registry => {
            let handle_event = registry::handle(json, map).await;
            publisher(sender, handle_event);
        }
        // 服务发现：根据service-name 获取所有的service
        RpcKind::Discovery => {
            let handle_event = discovery::handle(json, map).await;
            publisher(sender, handle_event);
        }
        // 获取所有的service-names
        RpcKind::DiscoveryNames => {
            let handle_event = discovery_names::handle(json, map).await;
            publisher(sender, handle_event);
        }
        // 服务下线
        RpcKind::Deregistry => {
            let handle_event = deregistry::handle(json, map).await;
            publisher(sender, handle_event);
        }
        // 服务检测
        RpcKind::ServiceCheck => {
            let handle_event = service_check::handle(json, map).await;
            publisher(sender, handle_event);
        }
    }
}
// 发布事件消息
fn publisher(sender: &mut Sender<InboundHandleEvent>, handle_event: InboundHandleEvent) {
    if let Err(result) = sender.send(handle_event) {
        error!("Publisher Event Error [{:?}]", result);
    }
}
