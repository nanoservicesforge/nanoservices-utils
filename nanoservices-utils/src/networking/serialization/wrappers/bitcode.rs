//! The wrapper for wrapping messages that are serialized using the `bitcode` crate for sending over a network.
//! WARNING: bitcode favours speed and compact size over stability, always test every contract before using it in production.
//! bitcode can even break between Rust versions.
use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
use std::io::{Read, Write};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use bitcode::{Encode, DecodeOwned};


/// The wrapper for wrapping messages that are serialized using the `bitcode` crate for sending over a network.
/// 
/// # Notes
/// - `bitcode` favours speed and compact size over stability, always test every contract before using it in production.
///
/// # Fields
/// * `header_bytes` - The bytes that represent the length of the contract bytes.
/// * `contract_bytes` - The bytes that represent the contract.
/// * `header` - The length of the contract bytes (in byte form).
/// * `contract` - The contract.
pub struct BitcodeContractWrapper<T: Encode + DecodeOwned> {
    header_bytes: Option<[u8; 4]>,
    contract_bytes: Option<Vec<u8>>,
    pub header: Option<u32>,
    pub contract: Option<T>,
}

impl <T: Encode + DecodeOwned> BitcodeContractWrapper<T> {

    /// Constructs a new `BitcodeContractWrapper` for when you are sending a contract.
    /// Refer to the `empty` function if you want to create a wrapper for receiving a contract.
    /// 
    /// # Arguments
    /// * `contract` - The contract to send.
    /// 
    /// # Returns
    /// * `Result<BitcodeContractWrapper<T>, NanoServiceError>` - The new `BitcodeContractWrapper`.
    pub fn new(contract: T) -> Result<Self, NanoServiceError> {
        let contract_bytes: Vec<u8> = bitcode::encode(&contract);
        let length = contract_bytes.len() as u32;
        let header_bytes = length.to_le_bytes();

        Ok(BitcodeContractWrapper {
            header_bytes: Some(header_bytes),
            contract_bytes: Some(contract_bytes),
            header: None,
            contract: None,
        })
    }

    /// Constructs an empty `BitcodeContractWrapper` for when you are receiving a contract. This
    /// means that everything is empty so bytes from the TCP connection can be read into the wrapper.
    /// For sending a contract, use the `new` function.
    /// 
    /// # Returns
    /// * `BitcodeContractWrapper<T>` - The empty `BitcodeContractWrapper`.
    pub fn empty() -> Self {
        BitcodeContractWrapper {
            header_bytes: None,
            contract_bytes: None,
            header: None,
            contract: None,
        }
    }

    /// Sends the contract over a blocking stream.
    /// 
    /// # Arguments
    /// * `stream` - The stream to send the contract over.
    pub fn blocking_send<X: Write>(&self, stream: &mut X) -> Result<(), NanoServiceError> {
        // extract the bytes to be sent
        let header_bytes = self.header_bytes.as_ref().unwrap();
        let contract_bytes = self.contract_bytes.as_ref().unwrap();

        // send the bytes to the stream
        stream.write_all(header_bytes).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        stream.write_all(&contract_bytes).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        Ok(())
    }

    /// Receives the contract over a blocking stream.
    /// 
    /// # Notes
    /// `self.header`, and `self.contract` will be populated with the values from the stream.
    /// 
    /// # Arguments
    /// * `stream` - The stream to receive the contract from.
    pub fn blocking_receive<X: Read>(&mut self, stream: &mut X) -> Result<(), NanoServiceError> {
        // extract the header to get the length of the contract
        let mut header_buffer = [0; 4];
        stream.read_exact(&mut header_buffer).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        let header = u32::from_le_bytes(header_buffer);

        // extract the contract, without allocating memory disproportionate to actual data
        let mut contract_buffer = Vec::with_capacity(header.min(1024) as usize);
        stream
            .take(header as u64)
            .read_to_end(&mut contract_buffer)
            .map_err(|e| {
                NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
            })?;
        if contract_buffer.len() != header as usize {
            return Err(NanoServiceError::new(
                "Unexpected EOF".to_owned(),
                NanoServiceErrorStatus::BadRequest,
            ));
        }
        self.header = Some(header);
        self.contract = Some(bitcode::decode::<T>(&contract_buffer).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?);
        Ok(())
    }

    /// Sends the contract over an async stream.
    /// 
    /// # Arguments
    /// * `stream` - The stream to send the contract over.
    pub async fn async_send<X: AsyncWriteExt + std::marker::Unpin>(&self, stream: &mut X) -> Result<(), NanoServiceError> {
        // extract the bytes to be sent
        let header_bytes = self.header_bytes.as_ref().unwrap();
        let contract_bytes = self.contract_bytes.as_ref().unwrap();

        stream.write_all(header_bytes).await.map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        stream.write_all(&contract_bytes).await.map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        Ok(())
    }

    /// Receives the contract over an async stream.
    /// 
    /// # Notes
    /// `self.header`, and `self.contract` will be populated with the values from the stream.
    /// 
    /// # Arguments
    /// * `stream` - The stream to receive the contract from.
    pub async fn async_receive<X: AsyncReadExt + std::marker::Unpin>(&mut self, stream: &mut X) -> Result<(), NanoServiceError> {
        // extract the header to get the length of the contract
        let mut header_buffer = [0; 4];
        stream.read_exact(&mut header_buffer).await.map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        let header = u32::from_le_bytes(header_buffer);

        // extract the contract, without allocating memory disproportionate to actual data
        let mut contract_buffer = Vec::with_capacity(header.min(1024) as usize);
        stream
            .take(header as u64)
            .read_to_end(&mut contract_buffer)
            .await
            .map_err(|e| {
                NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
            })?;
        if contract_buffer.len() != header as usize {
            return Err(NanoServiceError::new(
                "Unexpected EOF".to_owned(),
                NanoServiceErrorStatus::BadRequest,
            ));
        }
        self.header = Some(header);
        self.contract = Some(bitcode::decode::<T>(&contract_buffer).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?);
        Ok(())
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    mod kernel {
        use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
        use serde::{Serialize, Deserialize};
        use bitcode::{Encode, Decode};
        use crate::create_bitcode_contract_handler;

        #[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Encode, Decode)]
        pub struct ContractOne{
            pub name: String,
            pub age: i32,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
        pub struct ContractTwo;

        #[derive(Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
        pub struct ContractThree;

        create_bitcode_contract_handler!(
            ContractHandler, 
            ContractOne, 
            ContractTwo, 
            ContractThree
        );
    }

    mod server {
        use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
        use super::kernel::ContractHandler;
        use super::kernel::ContractOne;
        use super::kernel::ContractTwo;
        use super::super::BitcodeContractWrapper;
        use crate::register_contract_routes;

        use tokio::net::TcpListener;


        async fn handle_test_contract_one(mut contract: ContractOne) -> Result<ContractOne, NanoServiceError> {
            contract.age += 1;
            Ok(contract)
        }

        async fn handle_test_contract_two(contract: ContractTwo) -> Result<ContractTwo, NanoServiceError> {
            Ok(contract)
        }

        register_contract_routes!(
            ContractHandler, 
            handle_contract, 
            ContractOne => handle_test_contract_one, 
            ContractTwo => handle_test_contract_two
        );

        pub async fn tcp_server(addr: &str) {
            let listener = TcpListener::bind(addr).await.unwrap();

            while let Ok((mut socket, _)) = listener.accept().await {
                let mut recieving_wrapper = BitcodeContractWrapper::<ContractHandler>::empty();
                recieving_wrapper.async_receive(&mut socket).await.unwrap();
                let contract = recieving_wrapper.contract.unwrap();
                let response = match handle_contract(contract).await {
                    Ok(response) => response,
                    Err(e) => {
                        ContractHandler::NanoServiceError(e)
                    }
                
                };
                let sending_wrapper = BitcodeContractWrapper::new(response).unwrap();
                sending_wrapper.async_send(&mut socket).await.unwrap();
                break;
            }
        }
    }

    use kernel::{ContractHandler, ContractOne};
    use server::tcp_server;

    use tokio::runtime::Builder;

    #[test]
    fn test_bitcode_contract_wrapper_constructor() {
        let contract = ContractOne {
            name: "John".to_string(),
            age: 32,
        };
        let wrapper = BitcodeContractWrapper::new(contract.clone()).unwrap();

        // test the general contents
        assert_eq!(wrapper.header_bytes.is_some(), true);
        assert_eq!(wrapper.contract_bytes.is_some(), true);
        assert_eq!(wrapper.header.is_none(), true);
        assert_eq!(wrapper.contract.is_none(), true);
        // assert_eq!([16, 0, 0, 0], wrapper.header_bytes.unwrap());

        // test the deserialization and if the header is correct
        let deserialized_contract =
            bitcode::decode::<ContractOne>(&wrapper.contract_bytes.as_ref().unwrap()).unwrap();
        let deserialized_header = u32::from_le_bytes(wrapper.header_bytes.unwrap());
        assert_eq!(contract, deserialized_contract);
        assert_eq!(deserialized_header, wrapper.contract_bytes.unwrap().len() as u32);
    }

    #[test]
    fn test_async_send_over_tcp() {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let port = 8096;
            let address = format!("127.0.0.1:{}", port);
            let _server = tokio::spawn(tcp_server("127.0.0.1:8096"));
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let contract = ContractHandler::ContractOne(ContractOne {
                name: "John".to_string(),
                age: 32,
            });

            let mut wrapper = BitcodeContractWrapper::new(contract).unwrap();
            let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
            wrapper.async_send(&mut stream).await.unwrap();
            wrapper.async_receive(&mut stream).await.unwrap();

            let expected_contract = ContractHandler::ContractOne(ContractOne {
                name: "John".to_string(),
                age: 33,
            });
            assert_eq!(wrapper.contract.unwrap(), expected_contract);
        });
    }

    #[test]
    fn test_blocking_over_tcp() {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let port = 8097;
            let address = format!("127.0.0.1:{}", port);
            let _server = tokio::spawn(tcp_server("127.0.0.1:8097"));
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let contract = ContractHandler::ContractOne(ContractOne {
                name: "John".to_string(),
                age: 32,
            });

            let mut wrapper = BitcodeContractWrapper::new(contract).unwrap();
            let mut stream = std::net::TcpStream::connect(address).unwrap();
            wrapper.blocking_send(&mut stream).unwrap();
            wrapper.blocking_receive(&mut stream).unwrap();

            let expected_contract = ContractHandler::ContractOne(ContractOne {
                name: "John".to_string(),
                age: 33,
            });
            assert_eq!(wrapper.contract.unwrap(), expected_contract);
        });
    }
}
