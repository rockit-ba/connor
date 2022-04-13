use anyhow::Result;
use bytes::{Bytes, BytesMut};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt, TryStreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub type TcpReader = SplitStream<Framed<TcpStream, LengthDelimitedCodec>>;
pub type TcpWriter = SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>;


pub struct TcpClient {
    reader: TcpReader,
    writer: TcpWriter,
}

impl TcpClient {
    pub async fn new(connect: &str) -> Result<Self> {
        let tcp_stream = TcpStream::connect(connect).await?;

        let transport = Framed::new(tcp_stream, LengthDelimitedCodec::new());

        let (writer, reader) = transport.split();
        Ok(TcpClient {
            reader,
            writer
        })
    }

    pub async fn read(&mut self) -> Option<BytesMut> {
        match self.reader.try_next().await {
            Ok(ele) => {
                ele
            }
            Err(err) => {
                println!("接收响应失败：{:?}", err);
                None
            }
        }

    }

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






