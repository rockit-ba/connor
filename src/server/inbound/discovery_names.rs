//! 获取所有的service-names

use crate::models::request::DiscoveryServiceNamesRequest;
use crate::models::{InboundHandleSingleEvent, RpcCodec};
use crate::server_bootstrap::ServersMap;
use tracing::info;

pub async fn handle(json: &str, map: ServersMap) -> InboundHandleSingleEvent {
    let service_names_request = DiscoveryServiceNamesRequest::from_json(json);
    info!("inbound data [ {:?} ]", &service_names_request);
    let service_names;
    {
        let map = map.read();
        service_names = map.keys().cloned().collect();
    }

    InboundHandleSingleEvent::ServiceNamesResp { service_names }
}
