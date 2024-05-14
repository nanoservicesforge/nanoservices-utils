//! This code is currently parked for now to enable a release for the layered lib.
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::future::Future;
use std::pin::Pin;
use std::fmt::Debug;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::Framed;
use tokio_util::codec::{Decoder, Encoder};
use tokio::net::TcpStream;
use futures::{sink::SinkExt, StreamExt};

use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
use crate::networking::serialization::codec::BincodeCodec;


// pub struct TcpToWasmProxy<T: DeserializeOwned + Debug + Serialize> {
pub struct TcpToWasmProxy {
    pub address: String,
    pub wasm_path: String,
    // pub handler: T,
    // pub handle_func: fn(T) -> Pin<Box<dyn Future<Output = Result<T, NanoServiceError>> + Send>>,
}


// impl <T: DeserializeOwned + Debug + Serialize> TcpToWasmProxy<T> {
impl TcpToWasmProxy {
    pub fn new(address: String, wasm_path: String, 
        // handler: T, 
        //handle_func: fn(T) -> Pin<Box<dyn Future<Output = Result<T, NanoServiceError>> + Send>>
    ) -> Self {
        TcpToWasmProxy {
            address,
            wasm_path,
            // handler,
            // handle_func
        }
    }

    pub async fn start<T: DeserializeOwned + Debug + Serialize>(&self) -> Result<(), NanoServiceError> {
        // start the wasm server
        let mut child = Command::new("wasmtime")
        .arg(self.wasm_path.as_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start WASM process");

        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        let stdout = child.stdout.as_mut().expect("Failed to open stdout");
        let mut reader = BufReader::new(stdout);

        // start the tcp server
        let listener = TcpListener::bind(self.address.clone()).await.map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::Unknown)
        })?;

        while let Ok((socket, _)) = listener.accept().await {
            let mut framed = Framed::new(socket, BincodeCodec::<T>::new());
    
            while let Some(result) = framed.next().await {
                match result {
                    Ok(data) => {
                        println!("Received: {:?}", data);
                        // message type 1 is the contract, will add things like type 2 for data storage later
                        let message_type: u32 = 1;
                        let message_type_bytes = message_type.to_be_bytes();

                        // pack the message with the message type
                        let buf = bincode::serialize(&data).unwrap();
                        let mut encoded_message: Vec<u8> = Vec::with_capacity(message_type_bytes.len() + buf.len());
                        encoded_message.extend_from_slice(&message_type_bytes);
                        encoded_message.extend_from_slice(&buf);

                        // Send the message to the child process
                        stdin.write_all(&encoded_message).unwrap();
                        stdin.write_all(b"\n").unwrap();
                        stdin.flush().unwrap();

                        // Read the response for each message
                        let mut output = Vec::new();
                        reader.read_until(b'\n', &mut output).unwrap();

                        // process the response
                        let (type_prefix, message_data) = output.split_at(4);
                        let _request_type = u32::from_be_bytes([type_prefix[0], type_prefix[1], type_prefix[2], type_prefix[3]]);
                        let response: T = bincode::deserialize(&message_data).unwrap();

                        // return the response via TCP without any processing
                        framed.send(response).await.unwrap();
                        break;
                    },
                    Err(e) => {
                        eprintln!("Error processing data: {}", e);
                        break;
                    }
                }
            }
        }

        // loop {
        //     // Asynchronously wait for an inbound socket.
        //     let (mut socket, _) = listener.accept().await.map_err(|e| {
        //         NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::Unknown)
        //     })?;

        //     let mut buf = vec![0; 1024];

        //     // In a loop, read data from the socket and write the data back.
        //     loop {
        //         let n = socket
        //             .read(&mut buf)
        //             .await
        //             .expect("failed to read data from socket");

        //         if n == 0 {
        //             break
        //         }

        //     }

        //     // message type 1 is the contract, will add things like type 2 for data storage later
        //     let message_type: u32 = 1;
        //     let message_type_bytes = message_type.to_be_bytes();

        //     // pack with message type
        //     let mut encoded_message: Vec<u8> = Vec::with_capacity(message_type_bytes.len() + buf.len());
        //     encoded_message.extend_from_slice(&message_type_bytes);
        //     encoded_message.extend_from_slice(&buf);

        //     // Send the message to the child process
        //     stdin.write_all(&encoded_message).unwrap();
        //     stdin.write_all(b"\n").unwrap();
        //     stdin.flush().unwrap();

        //     // Read the response for each message
        //     let mut output = Vec::new();
        //     reader.read_until(b'\n', &mut output).unwrap();

        //     // process the response
        //     let (type_prefix, message_data) = output.split_at(4);
        //     let _request_type = u32::from_be_bytes([type_prefix[0], type_prefix[1], type_prefix[2], type_prefix[3]]);

        //     // return the response via TCP without any processing
        //     socket.write_all(&message_data).await.unwrap();
        // }
        Ok(())
        
    }
}
