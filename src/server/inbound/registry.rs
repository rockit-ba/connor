//! 服务注册

use crate::models::{InboundHandleEvent,RpcCodec};
use crate::server_bootstrap::ServersMap;
use tracing::info;
use crate::models::request::RegistryRequest;

/// 请求处理
///
/// 返回响应的事件
pub async fn handle(json: &str, map: ServersMap) -> InboundHandleEvent {
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

        info!("service lists [ {:?} ]", &servers);

        // 返回服务注册的事件
        InboundHandleEvent::ServiceRegistry {success: true}
    }
}
