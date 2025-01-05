//! Defines the TCP framing for the bincode serialization format.
use tokio_util::codec::{Decoder, Encoder};
use bytes::{BufMut, BytesMut};
use std::{io, marker::PhantomData};
use serde::Serialize;
use revision::Revisioned;


/// A codec that serializes and deserializes data using the bincode format for framing.
pub struct VersionedBincodeCodec<T> {
    phantom: PhantomData<T>,
}

impl<T> VersionedBincodeCodec<T> {
    pub fn new() -> Self {
        VersionedBincodeCodec { phantom: PhantomData }
    }
}

impl<T> Decoder for VersionedBincodeCodec<T> 
where
    T: serde::de::DeserializeOwned + Revisioned,
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

impl<T> Encoder<T> for VersionedBincodeCodec<T> 
where
    T: Serialize + Revisioned,
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




// #[cfg(test)]
// mod tests {

//     use super::*;

//     use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
//     use serde::{Serialize, Deserialize};
//     use tokio_util::codec::Framed;
//     use futures::{sink::SinkExt, StreamExt};
//     use crate::networking::serialization::codec::BincodeCodec;
//     use revision::revisioned;
//     use revision::Error;
//     use tokio_util::codec::Decoder;
//     use crate::register_contract_routes;
//     use bytes::{BufMut, BytesMut};
//     use crate::networking::tcp::client::send_data_contract_over_tcp;

//     // The test structure is at revision 3.
//     #[derive(Debug, PartialEq, Serialize, Deserialize)]
//     #[revisioned(revision = 3)]
//     pub struct ContractOne {
//         pub a: u32,
//         #[revision(start = 2, end = 3, convert_fn = "convert_b")]
//         pub b: u8,
//         #[revision(start = 3)]
//         pub c: u64,
//         #[revision(start = 3, default_fn = "default_c")]
//         pub d: String,
//     }

//     impl ContractOne {
//         // Used to set the default value for a newly added field.
//         fn default_c(_revision: u16) -> String {
//             "test_string".to_owned()
//         }
//         // Used to convert the field from an old revision to the latest revision
//         fn convert_b(&mut self, _revision: u16, value: u8) -> Result<(), Error> {
//             self.c = value as u64;
//             Ok(())
//         }
//     }

//     #[derive(Debug, PartialEq, Serialize, Deserialize)]
//     #[revisioned(revision = 3)]
//     pub struct ContractTwo {
//         pub a: u32,
//         #[revision(start = 2, end = 3, convert_fn = "convert_b")]
//         pub b: u8,
//         #[revision(start = 3)]
//         pub c: u64,
//         #[revision(start = 3, default_fn = "default_c")]
//         pub d: String,
//     }

//     impl ContractTwo {
//         // Used to set the default value for a newly added field.
//         fn default_c(_revision: u16) -> String {
//             "test_string".to_owned()
//         }
//         // Used to convert the field from an old revision to the latest revision
//         fn convert_b(&mut self, _revision: u16, value: u8) -> Result<(), Error> {
//             self.c = value as u64;
//             Ok(())
//         }
//     }

//     #[derive(Debug, PartialEq, Serialize, Deserialize)]
//     #[revisioned(revision = 1)]
//     pub enum ContractHandler {
//         #[revision(start = 1)]
//         ContractOne(ContractOne),
//         #[revision(start = 1)]
//         ContractTwo(ContractTwo),
//         Error(NanoServiceError),
//     }

//     async fn handle_test_contract_one(mut contract: ContractOne) -> Result<ContractOne, NanoServiceError> {
//         contract.a += 1;
//         Ok(contract)
//     }

//     async fn handle_test_contract_two(mut contract: ContractTwo) -> Result<ContractTwo, NanoServiceError> {
//         contract.a += 2;
//         Ok(contract)
//     }

//     register_contract_routes!(
//         ContractHandler,
//         handle_contract,
//         ContractOne => handle_test_contract_one, 
//         ContractTwo => handle_test_contract_two
//     );

//     async fn run_tcp_server(addr: String) {
//         let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
//         while let Ok((socket, _)) = listener.accept().await {
//             let mut framed = Framed::new(socket, VersionedBincodeCodec::<ContractHandler>::new());

//             while let Some(result) = framed.next().await {
//                 match result {
//                     Ok(data) => {
//                         let response = handle_contract(data).await.unwrap();
//                         framed.send(response).await.unwrap();
//                         break;
//                     },
//                     Err(e) => {
//                         eprintln!("Error processing data: {}", e);
//                         break;
//                     }
//                 }
//             }
//         }
//     }

//     #[test]
//     fn test_bincode_codec() {
//         let mut codec = BincodeCodec::<ContractHandler>::new();
//         let test_contract = ContractHandler::ContractOne(
//             ContractOne {
//                 a: 42,
//                 b: 1,
//                 c: 2,
//                 d: "hello".to_string(),
//             }
//         );

//         let encoded = bincode::serialize(&test_contract).unwrap();
//         let mut buf = BytesMut::with_capacity(encoded.len());
//         buf.put_slice(&encoded);
//         let decoded = codec.decode(&mut buf).unwrap().unwrap();
//         assert_eq!(test_contract, decoded);
//     }

//     #[test]
//     fn test_tcp_framing_contract_one() {
//         let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
//             .worker_threads(1)
//             .enable_all()
//             .build()
//             .unwrap();
//         let port = 8091;
//         let addr = format!("0.0.0.0:{}", port);
//         let server_handle = tokio_runtime.spawn(run_tcp_server(addr.clone()));
//         let data = ContractHandler::ContractOne(
//             ContractOne {
//                 a: 42,
//                 b: 1,
//                 c: 2,
//                 d: "hello".to_string(),
//             }
//         );
//         // send data to the server
//         tokio_runtime.block_on(async {
//             let response = send_data_contract_over_tcp(data, &addr).await.unwrap();
//             assert_eq!(response, ContractHandler::ContractOne(
//                 ContractOne {
//                     a: 43,
//                     b: 1,
//                     c: 2,
//                     d: "hello".to_string(),
//                 }
//             ));
//         });
//         std::mem::drop(server_handle);
//     }

//     #[test]
//     fn test_tcp_framing_contract_two() {
//         let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
//             .worker_threads(1)
//             .enable_all()
//             .build()
//             .unwrap();
//         let port = 8093;
//         let addr = format!("0.0.0.0:{}", port);
//         let server_handle = tokio_runtime.spawn(run_tcp_server(addr.clone()));

//         let data = ContractHandler::ContractTwo(
//             ContractTwo {
//                 a: 42,
//                 b: 1,
//                 c: 2,
//                 d: "hello".to_string(),
//             }
//         );
//         // send data to the server
//         tokio_runtime.block_on(async {
//             let response = send_data_contract_over_tcp(data, &addr).await.unwrap();
//             assert_eq!(response, ContractHandler::ContractTwo(
//                 ContractTwo {
//                     a: 44,
//                     b: 1,
//                     c: 2,
//                     d: "hello".to_string(),
//                 }
//             ));
//         });
//         std::mem::drop(server_handle);
//     }

// }