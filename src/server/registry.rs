//! 服务注册

use crate::models::{RegistryRequest, RegistryResponse, RpcCodec, TcpWriter};
use crate::server_bootstrap::ServersMap;
use bytes::Bytes;
use futures::SinkExt;
use tracing::{error, info};

pub async fn handle(json: &str, writer: &mut TcpWriter, map: ServersMap) {
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
    }

    broadcast().await;

    response(&registry_req,writer).await;

}

async fn broadcast() {

}


async fn response(registry_req: &RegistryRequest,writer: &mut TcpWriter) {
    // 响应注册服务
    let registry_response = RegistryResponse::new(true, &registry_req.service.name);
    let content = registry_response.to_json();
    if let Err(err) = writer
        .send(Bytes::copy_from_slice(content.as_bytes()))
        .await
    {
        error!("{:?}", err);
    }
}
