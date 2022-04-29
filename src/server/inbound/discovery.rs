//! 服务发现：根据service-name 获取所有的service

use crate::models::{InboundHandleEvent, RpcCodec};
use crate::server_bootstrap::ServersMap;
use tracing::info;
use crate::models::request::DiscoveryRequest;

pub async fn handle(json: &str, map: ServersMap) -> InboundHandleEvent {
    let discovery_req = DiscoveryRequest::from_json(json);
    info!("inbound data [ {:?} ]", &discovery_req);
    let mut services = None;
    {
        let map = map.read();
        if let Some(lists) = map.get(&discovery_req.service_name) {
            services = Some(lists.clone());
        }
    }
    // 返回服务注册的事件
    InboundHandleEvent::ServiceDiscoveryResp {
        service_name: discovery_req.service_name.clone(),
        services,
    }
}
