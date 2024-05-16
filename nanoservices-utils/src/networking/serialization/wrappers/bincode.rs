//! The wrapper for wrapping messages that are serialized using the `bincode` crate for sending over a network.
use serde::{Serialize, de::DeserializeOwned};
use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
use std::io::{Read, Write};
use tokio::io::{AsyncWriteExt, AsyncReadExt};


/// The wrapper for wrapping messages that are serialized using the `bincode` crate for sending over a network.
/// 
/// # Fields
/// * `header_bytes` - The bytes of the header that contains the length of the contract.
/// * `contract_bytes` - The bytes of the contract.
/// * `header` - The length of the contract (in byte form).
/// * `contract` - The contract.
pub struct BincodeContractWrapper<T: Serialize + DeserializeOwned> {
    header_bytes: Option<[u8; 4]>,
    contract_bytes: Option<Vec<u8>>,
    pub header: Option<u32>,
    pub contract: Option<T>,
}

impl <T: Serialize + DeserializeOwned> BincodeContractWrapper<T> {

    /// Constructs a new `BincodeContractWrapper` for when you are sending a contract.
    /// Refer to the `empty` function if you want to create a wrapper for receiving a contract.
    /// 
    /// # Arguments
    /// * `contract` - The contract to send.
    /// 
    /// # Returns
    /// * `Result<BincodeContractWrapper<T>, NanoServiceError>` - The new `BincodeContractWrapper`.
    pub fn new(contract: T) -> Result<Self, NanoServiceError> {
        let contract_bytes: Vec<u8> = bincode::serialize(&contract).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        let length = contract_bytes.len() as u32;
        let header_bytes_buffer: Vec<u8> = bincode::serialize(&length).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        
        if header_bytes_buffer.len() != 4 {
            return Err(NanoServiceError::new("Header bytes length is not 4.".to_string(), NanoServiceErrorStatus::BadRequest));
        }
        let header_bytes: [u8; 4] = [
            header_bytes_buffer[0],
            header_bytes_buffer[1],
            header_bytes_buffer[2],
            header_bytes_buffer[3],
        ];
        Ok(BincodeContractWrapper {
            header_bytes: Some(header_bytes),
            contract_bytes: Some(contract_bytes),
            header: None,
            contract: None,
        })
    }

    /// Constructs an empty `BincodeContractWrapper` for when you are receiving a contract. This
    /// means that everything is empty so bytes from the TCP connection can be read into the wrapper.
    /// For sending a contract, use the `new` function.
    /// 
    /// # Returns
    /// * `BincodeContractWrapper<T>` - The empty `BincodeContractWrapper`.
    pub fn empty() -> Self {
        BincodeContractWrapper {
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
        let header_bytes = self.header_bytes.unwrap();
        let contract_bytes = self.contract_bytes.as_ref().unwrap();
        stream.write_all(&header_bytes).map_err(|e| {
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
        let mut header_buffer = [0; 4];
        stream.read_exact(&mut header_buffer).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        let header = bincode::deserialize::<u32>(&header_buffer).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        let mut contract_buffer = vec![0; header as usize];
        stream.read_exact(&mut contract_buffer).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        self.header = Some(header);
        self.contract = Some(bincode::deserialize::<T>(&contract_buffer).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?);
        Ok(())
    }

    /// Sends the contract over an async stream.
    /// 
    /// # Arguments
    /// * `stream` - The stream to send the contract over.
    pub async fn async_send<X: AsyncWriteExt + std::marker::Unpin>(&self, stream: &mut X) -> Result<(), NanoServiceError> {
        let header_bytes = self.header_bytes.unwrap();
        let contract_bytes = self.contract_bytes.as_ref().unwrap();
        stream.write_all(&header_bytes).await.map_err(|e| {
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
        let mut header_buffer = [0; 4];
        stream.read_exact(&mut header_buffer).await.map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        let header = bincode::deserialize::<u32>(&header_buffer).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        let mut contract_buffer = vec![0; header as usize];
        stream.read_exact(&mut contract_buffer).await.map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?;
        self.header = Some(header);
        self.contract = Some(bincode::deserialize::<T>(&contract_buffer).map_err(|e| {
            NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
        })?);
        Ok(())
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    mod kernel {
        use crate::create_contract_handler;
        use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
        use serde::{Serialize, Deserialize};

        #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
        pub struct ContractOne{
            pub name: String,
            pub age: i32,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub struct ContractTwo;

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub struct ContractThree;

        create_contract_handler!(
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
        use super::super::BincodeContractWrapper;
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
                let mut recieving_wrapper = BincodeContractWrapper::<ContractHandler>::empty();
                recieving_wrapper.async_receive(&mut socket).await.unwrap();
                let contract = recieving_wrapper.contract.unwrap();
                let response = match handle_contract(contract).await {
                    Ok(response) => response,
                    Err(e) => {
                        ContractHandler::NanoServiceError(e)
                    }
                
                };
                let sending_wrapper = BincodeContractWrapper::new(response).unwrap();
                sending_wrapper.async_send(&mut socket).await.unwrap();
                break;
            }
        }
    }

    use kernel::{ContractHandler, ContractOne};
    use server::tcp_server;

    use tokio::runtime::Builder;

    #[test]
    fn test_bincode_contract_wrapper_constructor() {
        let contract = ContractOne {
            name: "John".to_string(),
            age: 32,
        };
        let wrapper = BincodeContractWrapper::new(contract.clone()).unwrap();

        // test the general contents
        assert_eq!(wrapper.header_bytes.is_some(), true);
        assert_eq!(wrapper.contract_bytes.is_some(), true);
        assert_eq!(wrapper.header.is_none(), true);
        assert_eq!(wrapper.contract.is_none(), true);
        assert_eq!([16, 0, 0, 0], wrapper.header_bytes.unwrap());

        // test the deserialization and if the header is correct
        let deserialized_contract = bincode::deserialize::<ContractOne>(&wrapper.contract_bytes.as_ref().unwrap()).unwrap();
        let deserialized_header = bincode::deserialize::<u32>(&wrapper.header_bytes.unwrap()).unwrap();
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
            let port = 8094;
            let address = format!("127.0.0.1:{}", port);
            let _server = tokio::spawn(tcp_server("127.0.0.1:8094"));
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let contract = ContractHandler::ContractOne(ContractOne {
                name: "John".to_string(),
                age: 32,
            });

            let mut wrapper = BincodeContractWrapper::new(contract).unwrap();
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
            let port = 8095;
            let address = format!("127.0.0.1:{}", port);
            let _server = tokio::spawn(tcp_server("127.0.0.1:8095"));
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let contract = ContractHandler::ContractOne(ContractOne {
                name: "John".to_string(),
                age: 32,
            });

            let mut wrapper = BincodeContractWrapper::new(contract).unwrap();
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
