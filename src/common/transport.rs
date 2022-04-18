use crate::common::models::{TcpReader, TcpWriter};
use anyhow::Result;
use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt, TryStreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[allow(dead_code)]
pub struct TcpClient {
    reader: TcpReader,
    writer: TcpWriter,
}

impl TcpClient {
    #[allow(dead_code)]
    pub async fn new(connect: &str) -> Result<Self> {
        let tcp_stream = TcpStream::connect(connect).await?;

        let transport = Framed::new(tcp_stream, LengthDelimitedCodec::new());

        let (writer, reader) = transport.split();
        Ok(TcpClient { reader, writer })
    }

    #[allow(dead_code)]
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
