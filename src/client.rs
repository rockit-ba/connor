use crate::models::{TcpReader, TcpWriter};
use anyhow::Result;
use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt, TryStreamExt};
use parking_lot::RwLock;
use std::sync::Arc;
use std::thread::sleep;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, info};

type PeerClient = Arc<RwLock<Vec<TcpClient>>>;
/// 集群客户端集合
#[allow(dead_code)]
#[derive(Clone)]
pub struct PeerCluster {
    pub clients: PeerClient,
}
impl PeerCluster {
    pub async fn init(&mut self, cluster_addr: &Vec<String>) {
        for addr in cluster_addr {
            loop {
                match TcpClient::new(addr).await {
                    Ok(client) => {
                        info!("Connect peer [{}] success", addr);
                        self.clients.write().push(client);
                        break;
                    }
                    Err(e) => {
                        error!("Connect peer [{}] failed, err: [{:?}]", addr, e);
                        // 出错重试
                        sleep(tokio::time::Duration::from_secs(5));
                    }
                }
            }
        }
    }
}

/// 连接其它集群实例的客户端
#[allow(dead_code)]
pub struct TcpClient {
    reader: TcpReader,
    writer: TcpWriter,
}

impl TcpClient {
    #[allow(dead_code)]
    /// 根据一个地址创建一个可读写的客户端
    pub async fn new(connect: &str) -> Result<Self> {
        info!("Connect peer [{}] ....", connect);
        // tokio::time::timeout(TcpStream::connect(...))
        let tcp_stream = TcpStream::connect(connect).await?;

        let transport = Framed::new(tcp_stream, LengthDelimitedCodec::new());
        let (writer, reader) = transport.split();
        Ok(TcpClient { reader, writer })
    }

    #[allow(dead_code)]
    /// 读取数据
    pub async fn read(&mut self) -> Option<BytesMut> {
        match self.reader.try_next().await {
            Ok(ele) => ele,
            Err(err) => {
                println!("接收响应失败：{:?}", err);
                None
            }
        }
    }

    #[allow(dead_code)]
    /// 写入数据
    pub async fn write(&mut self, data: Bytes) {
        match self.writer.send(data).await {
            Ok(_) => {
                println!("数据发送成功");
            }
            Err(err) => {
                println!("数据发送失败：{:?}", err);
            }
        }
    }
}
