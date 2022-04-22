//! 服务发现：根据service-name 获取所有的service

use crate::models::{DiscoveryRequest, DiscoveryResponse, RpcCodec, TcpWriter};
use crate::server_bootstrap::ServersMap;
use bytes::Bytes;
use futures::SinkExt;
use tracing::{error, info};

pub async fn handle(json: &str, writer: &mut TcpWriter, map: ServersMap) {
    let discovery_req = DiscoveryRequest::from_json(json);
    info!("解码入站数据 {:?}", &discovery_req);
    let mut services = None;
    {
        let map = map.read();
        if let Some(lists) = map.get(&discovery_req.service_name) {
            services = Some(lists.clone());
        }
    }
    let discovery_response = DiscoveryResponse::new(&discovery_req.service_name, services);
    let content = discovery_response.to_json();

    info!("回送service list：{}", &content);
    if let Err(err) = writer
        .send(Bytes::copy_from_slice(content.as_bytes()))
        .await
    {
        error!("{:?}", err);
    }
}
