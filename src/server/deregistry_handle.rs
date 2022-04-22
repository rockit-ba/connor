//!  服务下线

use crate::models::{DeregistryRequest, RpcCodec, TcpWriter};
use crate::server_bootstrap::ServersMap;
use tracing::info;

pub async fn handle(json: &str, _writer: &mut TcpWriter, map: ServersMap) {
    let deregistry_request = DeregistryRequest::from_json(json);
    info!("inbound data {:?}", &deregistry_request);
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
}
