//! Defines the TCP framing for the bincode serialization format.
use tokio_util::codec::{Decoder, Encoder};
use bytes::{BufMut, BytesMut};
use std::{io, marker::PhantomData};
use serde::Serialize;


/// A codec that serializes and deserializes data using the bincode format for framing.
pub struct BincodeCodec<T> {
    phantom: PhantomData<T>,
}

impl<T> BincodeCodec<T> {
    pub fn new() -> Self {
        BincodeCodec { phantom: PhantomData }
    }
}

impl<T> Decoder for BincodeCodec<T> 
where
    T: serde::de::DeserializeOwned,
{
    type Item = T;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        bincode::deserialize(&src[..]).map(Some).map_err(|e| {
            eprintln!("Decode failed: {:?}", e);
            io::Error::new(io::ErrorKind::Other, "deserialize failed")
        })
    }
}

impl<T> Encoder<T> for BincodeCodec<T> 
where
    T: Serialize,
{
    type Error = io::Error;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = bincode::serialize(&item).map_err(|e| {
            eprintln!("Encode failed: {:?}", e);
            io::Error::new(io::ErrorKind::Other, "serialize failed")
        })?;
        dst.reserve(encoded.len());
        dst.put_slice(&encoded);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tokio_util::codec::Decoder;
    use tokio_util::codec::Framed;
    use futures::{sink::SinkExt, StreamExt};

    #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
    struct TestStruct {
        field1: u32,
        field2: String,
    }

    #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
    struct VersionedTestStruct{
        field1: u32,
        field2: String,
    }

    async fn run_tcp_server(addr: String) {
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        while let Ok((socket, _)) = listener.accept().await {
            let mut framed = Framed::new(socket, BincodeCodec::<TestStruct>::new());

            while let Some(result) = framed.next().await {
                match result {
                    Ok(mut data) => {
                        data.field1 += 1;
                        framed.send(data).await.unwrap();
                        break;
                    },
                    Err(e) => {
                        eprintln!("Error processing data: {}", e);
                        break;
                    }
                }
            }
        }
    }

    #[test]
    fn test_bincode_codec() {
        let mut codec = BincodeCodec::<TestStruct>::new();
        let test_struct = TestStruct {
            field1: 42,
            field2: "hello".to_string(),
        };
        let encoded = bincode::serialize(&test_struct).unwrap();
        let mut buf = BytesMut::with_capacity(encoded.len());
        buf.put_slice(&encoded);
        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(test_struct, decoded);
    }

    #[test]
    fn test_tcp_framing() {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let port = 8090;
        let addr = format!("0.0.0.0:{}", port);
        let server_handle = tokio_runtime.spawn(run_tcp_server(addr.clone()));
        let data = TestStruct {
            field1: 42,
            field2: "hello".to_string(),
        };
        // send data to the server
        tokio_runtime.block_on(async {
            let stream = tokio::net::TcpStream::connect(&addr).await.unwrap();
            let mut framed = Framed::new(stream, BincodeCodec::<TestStruct>::new());
            framed.send(data).await.unwrap();
            let response = framed.next().await.unwrap().unwrap();
            assert_eq!(response.field1, 43);
        });
        std::mem::drop(server_handle);
    }

}
