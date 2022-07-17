use futures::{AsyncRead, AsyncWrite, AsyncWriteExt};
use libp2p::{request_response::RequestResponseCodec, core::{ProtocolName, upgrade::{read_length_prefixed, write_length_prefixed}}};
use async_trait::async_trait;
use std::io;

#[derive(Debug, Clone)]
pub struct Protocol;
impl ProtocolName for Protocol {
    fn protocol_name(&self) -> &[u8] {
        "/digital-fax/0.1.0".as_ref()
    }
}


#[derive(Debug, Clone)]
pub struct Ack;

#[derive(Debug, Clone)]
pub struct Codec;

#[async_trait]
impl RequestResponseCodec for Codec {
    type Protocol = Protocol;
    type Request = Vec<u8>;
    type Response= Ack;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send {
            let vec = read_length_prefixed(io, 1024).await?;
            if vec.is_empty() {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            Ok(vec)
        }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send 
    {
        let vec = read_length_prefixed(io, 1).await?;
        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        Ok(Ack)
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send {

        write_length_prefixed(io, req).await?;
        io.close().await?;

        Ok(())
        }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        _: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send {
            write_length_prefixed(io, vec![0]).await?;
        io.close().await?;

        Ok(())
        }
}