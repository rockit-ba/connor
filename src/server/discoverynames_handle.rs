//! 获取所有的service-names

use crate::models::{
    DiscoveryServiceNamesRequest, DiscoveryServiceNamesResponse, RpcCodec, TcpWriter,
};
use crate::server_bootstrap::ServersMap;
use bytes::Bytes;
use futures::SinkExt;
use tracing::{error, info};

pub async fn handle(json: &str, writer: &mut TcpWriter, map: ServersMap) {
    let service_names_request = DiscoveryServiceNamesRequest::from_json(json);
    info!("inbound data {:?}", &service_names_request);
    let service_names;
    {
        let map = map.read();
        service_names = map.keys().cloned().collect();
    }
    let names_response = DiscoveryServiceNamesResponse::new(service_names);
    let content = names_response.to_json();

    info!("response service names：{}", &content);
    if let Err(err) = writer
        .send(Bytes::copy_from_slice(content.as_bytes()))
        .await
    {
        error!("{:?}", err);
    }
}
