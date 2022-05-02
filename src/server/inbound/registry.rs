//! 服务注册

use crate::models::request::RegistryRequest;
use crate::models::{InboundHandleBroadcastEvent, RpcCodec};
use crate::server_bootstrap::ServersMap;
use tracing::info;

/// 请求处理
///
/// 返回此次注册的服务结构体
pub async fn handle(json: &str, map: ServersMap) -> InboundHandleBroadcastEvent {
    let registry_req = RegistryRequest::from_json(json);
    info!("inbound data [ {:?} ]", &registry_req);
    // 存储注册的服务
    let service = &registry_req.service;
    {
        let mut servers = map.write();
        match servers.get_mut(&service.name) {
            Some(list) => {
                list.push(service.clone());
            }
            None => {
                servers.insert(service.name.clone(), vec![service.clone()]);
            }
        }
    }
    InboundHandleBroadcastEvent::AddServiceResp {
        service_name: service.name.clone(),
        service_list: map.read().get(&service.name).unwrap().clone(),
    }
}
