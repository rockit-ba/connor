//! 获取所有的service-names

use crate::models::{InboundHandleEvent, RpcCodec};
use crate::server_bootstrap::ServersMap;
use tracing::info;
use crate::models::request::DiscoveryServiceNamesRequest;

pub async fn handle(json: &str, map: ServersMap) -> InboundHandleEvent {
    let service_names_request = DiscoveryServiceNamesRequest::from_json(json);
    info!("inbound data [ {:?} ]", &service_names_request);
    let service_names;
    {
        let map = map.read();
        service_names = map.keys().cloned().collect();
    }

    InboundHandleEvent::ServiceNamesResp { service_names }
}
