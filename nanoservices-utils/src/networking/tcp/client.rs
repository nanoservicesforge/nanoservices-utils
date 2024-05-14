//! Defines the TCP client for sending data contracts over the network.
use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use crate::networking::serialization::codec::BincodeCodec;
use futures::{sink::SinkExt, StreamExt};


/// Sends a data contract over TCP to the specified address.
/// 
/// # Arguments
/// * `contract` - The contract to send.
/// * `address` - The address to send the contract to.
/// 
/// # Returns
/// * `Result<T, NanoServiceError>` - The response from the server which is either the contract or an Error.
pub async fn send_data_contract_over_tcp<T>(contract: T, address: &str) -> Result<T, NanoServiceError> 
where 
    T: Serialize + DeserializeOwned,
{
    let stream = TcpStream::connect(address).await.map_err(|e| {
        NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
    })?;
    let mut framed = Framed::new(stream, BincodeCodec::<T>::new());
    framed.send(contract).await.map_err(|e| {
        NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
    })?;
    let response = match framed.next().await {
        Some(response) => response,
        None => return Err(NanoServiceError::new("No response from server.".to_string(), NanoServiceErrorStatus::BadRequest))
    };
    Ok(response.map_err(|e| {
        NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
    })?)
}


#[cfg(test)]
mod tests {

    mod kernel {
        use crate::create_contract_handler;
        use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
        use serde::{Serialize, Deserialize};

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub struct ContractOne;

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
        use crate::register_contract_routes;

        use tokio::net::TcpListener;
        use tokio_util::codec::Framed;
        use crate::networking::serialization::codec::BincodeCodec;
        use futures::{sink::SinkExt, StreamExt};


        async fn handle_test_contract_one(contract: ContractOne) -> Result<ContractOne, NanoServiceError> {
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

            while let Ok((socket, _)) = listener.accept().await {
                let mut framed = Framed::new(socket, BincodeCodec::<ContractHandler>::new());

                while let Some(result) = framed.next().await {
                    match result {
                        Ok(data) => {
                            let response = match handle_contract(data).await {
                                Ok(response) => response,
                                Err(e) => {
                                    ContractHandler::NanoServiceError(e)
                                }
                            
                            };
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
        }
    }

    use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
    use kernel::{ContractHandler, ContractOne, ContractThree, ContractTwo};
    use server::tcp_server;
    use crate::networking::tcp::client::send_data_contract_over_tcp;

    use tokio::runtime::Builder;

    #[test]
    fn test_send_over_tcp() {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let address = "127.0.0.1:8080";
            let _server = tokio::spawn(tcp_server(address));
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let contract = ContractHandler::ContractOne(ContractOne);
            let response = send_data_contract_over_tcp(contract, address).await.unwrap();
            assert_eq!(response.ContractOne().unwrap(), ContractOne);

            let contract_two = ContractHandler::ContractTwo(ContractTwo);
            let response_two = send_data_contract_over_tcp(contract_two, address).await.unwrap();
            assert_eq!(response_two.ContractTwo().unwrap(), ContractTwo);

            let contract_three: ContractHandler = ContractHandler::ContractThree(ContractThree);
            let response_three = send_data_contract_over_tcp(contract_three, address).await.unwrap();
            assert_eq!(response_three.NanoServiceError().unwrap(), NanoServiceError::new(
                "Received unknown contract type.".to_string(),
                NanoServiceErrorStatus::ContractNotSupported
            ));
        });
    }
}