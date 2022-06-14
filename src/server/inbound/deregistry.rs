//!  服务下线

use crate::models::request::DeregistryRequest;
use crate::models::{InboundHandleBroadcastEvent, RpcCodec};
use crate::server_bootstrap::ServersMap;
use tracing::info;

pub async fn handle(json: &str, map: ServersMap) -> InboundHandleBroadcastEvent {
    let deregistry_request = DeregistryRequest::from_json(json);
    let service_name = &deregistry_request.service_name;
    info!("inbound data [ {:?} ]", &deregistry_request);
    {
        if let Some(services) = map.write().get_mut(service_name) {
            *services = services
                .iter()
                .filter(|&service| service.id.ne(&deregistry_request.service_id))
                .cloned()
                .collect();
        }
    }
    {
        InboundHandleBroadcastEvent::RemoveServiceResp {
            service_name: service_name.clone(),
            service_list: match map.read().get(service_name) {
                None => {
                    vec![]
                }
                Some(list) => list.clone(),
            },
        }
    }
}
