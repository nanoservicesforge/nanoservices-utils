use tokio_util::codec::{Decoder, Encoder};
use bytes::{BufMut, BytesMut};
use std::{io, marker::PhantomData};
use serde::Serialize;

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
