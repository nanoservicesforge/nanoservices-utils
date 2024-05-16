//! Bitcode codec for tokio. Right now it cannot really be used as `Encode` and `Decode` traits do not play well
//! with tokio framing. If you want to send a message using `bitcode` for serialization you can do this using the
//! `BitcodeContractWrapper` struct in the `wrappers` module.
use tokio_util::codec::{Decoder, Encoder};
use bytes::{BufMut, BytesMut};
use std::{io, marker::PhantomData};
use bitcode::{DecodeOwned, Encode};


pub struct BitcodeCodec<T> {
    phantom: PhantomData<T>,
}

impl<T> BitcodeCodec<T> {
    pub fn new() -> Self {
        BitcodeCodec { phantom: PhantomData }
    }
}

impl<T> Decoder for BitcodeCodec<T> 
where
    T: DecodeOwned
{
    type Item = T;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        bitcode::decode(&src[..]).map(Some).map_err(|e| {
            eprintln!("Decode failed: {:?}", e);
            io::Error::new(io::ErrorKind::Other, "deserialize failed")
        })
    }
}

impl<T> Encoder<T> for BitcodeCodec<T> 
where
    T: Encode,
{
    type Error = io::Error;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = bitcode::encode(&item);
        dst.reserve(encoded.len());
        dst.put_slice(&encoded);
        Ok(())
    }
}