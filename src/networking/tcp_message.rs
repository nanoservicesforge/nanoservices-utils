// use tokio::net::TcpStream;
// use tokio::io::{AsyncWriteExt, AsyncReadExt};
// use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
// use crate::networking::contract::Contract;


// // macro_rules! wrap_contract {
// //     ($contract:expr) => {
// //         let wrapped_contract = WrappedContract::from_contract($contract);
// //     };
// // }


// pub async fn send<D, R, T: Contract<D, R>>(address: &str, data: T) -> Result<R, NanoServiceError> {
//     let mut stream = TcpStream::connect(address).await.map_err(|e| {
//         NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::Unknown)
//     })?;
//     let serialized = bitcode::encode(&data);
//     stream.write_all(&serialized).await.map_err(|e| {
//         NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::Unknown)
//     })?;

//     stream.flush().await.map_err(|e| {
//         NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::Unknown)
//     })?;

//     // Read response from the server
//     let mut response = Vec::new();
//     let mut buffer = vec![0; 1024]; // Buffer for reading chunks
//     loop {
//         let n = stream.read(&mut buffer).await.map_err(|e| {
//             NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::Unknown)
//         })?;
//         if n == 0 {
//             break; // End of stream
//         }
//         response.extend_from_slice(&buffer[..n]);
//     }

//     let response: T = bitcode::decode(&response).map_err(|e| {
//         NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::Unknown)
//     })?;
//     response.result()
// }


// // send message

// // reiceve message