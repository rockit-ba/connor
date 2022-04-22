//! 服务检测

use crate::models::{RpcCodec, ServiceCheckRequest, ServiceCheckResponse, TcpWriter};
use crate::server_bootstrap::ServersMap;
use bytes::Bytes;
use futures::SinkExt;
use tracing::{error, info};

pub async fn handle(json: &str, writer: &mut TcpWriter, map: ServersMap) {
    let check_request = ServiceCheckRequest::from_json(json);
    info!("inbound data {:?}", &check_request);
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
    let response = ServiceCheckResponse { service_id };
    let content = response.to_json();
    info!("response service-id：{}", &content);
    if let Err(err) = writer
        .send(Bytes::copy_from_slice(content.as_bytes()))
        .await
    {
        error!("{:?}", err);
    }
}
