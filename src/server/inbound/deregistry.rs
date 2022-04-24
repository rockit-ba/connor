//!  服务下线

use crate::models::{InboundHandleEvent, RpcCodec};
use crate::server_bootstrap::ServersMap;
use tracing::info;
use crate::models::request::DeregistryRequest;

pub async fn handle(json: &str, map: ServersMap) -> InboundHandleEvent {
    let deregistry_request = DeregistryRequest::from_json(json);
    info!("inbound data [ {:?} ]", &deregistry_request);
    {
        let mut map = map.write();
        if let Some(services) = map.get_mut(&deregistry_request.service_name) {
            *services = services
                .iter()
                .filter(|&service| service.id.ne(&deregistry_request.service_id))
                .cloned()
                .collect();
        }
    }
    InboundHandleEvent::ServiceOfOut {
        service_name: deregistry_request.service_name.clone(),
        service_id: deregistry_request.service_id.clone(),
    }
}
