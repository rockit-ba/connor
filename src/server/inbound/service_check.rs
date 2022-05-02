//! 服务检测

use crate::models::request::ServiceCheckRequest;
use crate::models::{InboundHandleSingleEvent, RpcCodec};
use crate::server_bootstrap::ServersMap;
use tracing::info;

pub async fn handle(json: &str, map: ServersMap) -> InboundHandleSingleEvent {
    let check_request = ServiceCheckRequest::from_json(json);
    info!("inbound data [ {:?} ]", &check_request);
    let service_id: String;
    {
        let map = map.read();
        service_id = map
            .values()
            .flat_map(|ele| {
                ele.iter()
                    .map(|ele| ele.id.clone())
                    .filter(|ele| ele.eq(&check_request.service_id))
            })
            .collect();
    }
    info!("{}", &service_id);
    InboundHandleSingleEvent::ServiceCheckResp { service_id }
}
