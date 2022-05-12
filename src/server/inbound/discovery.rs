//! 服务发现：根据service-name 获取所有的service

use crate::models::request::DiscoveryRequest;
use crate::models::{InboundHandleSingleEvent, RpcCodec};
use crate::server_bootstrap::ServersMap;
use tracing::info;

pub async fn handle(json: &str, map: ServersMap) -> InboundHandleSingleEvent {
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
    InboundHandleSingleEvent::ServiceDiscoveryResp {
        service_name: discovery_req.service_name.clone(),
        services,
    }
}
