

#[macro_export]
macro_rules! create_contract_handler {
    ($enum_name:ident, $( $variant:ident ),*) => {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub enum $enum_name {
            $( $variant($variant), )+
        }

        impl $enum_name {
            $(
                #[allow(non_snake_case)]
                pub fn $variant(self) -> Result<$variant, NanoServiceError> {
                    match self {
                        $enum_name::$variant(inner) => Ok(inner),
                        _ => Err(NanoServiceError::new(
                                format!("Expected variant: {}", stringify!($variant)),
                                NanoServiceErrorStatus::BadRequest
                            )
                        ),
                    }
                }
            )+
        }

        impl $enum_name {

            pub async fn send_contract_over_tcp(self, address: &str) -> Result<Self, NanoServiceError> {
                let serialized = bincode::serialize(&self).map_err(|e| {
                    NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
                })?;
                let mut stream = TcpStream::connect(address).await.map_err(|e| {
                    NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
                })?;
                stream.write_all(&serialized).await.map_err(|e| {
                    NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
                })?;
                stream.flush().await.map_err(|e| {
                    NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
                })?;

                // Read response from the server
                let mut response = Vec::new();
                let mut buffer = vec![0; 1024]; // Buffer for reading chunks
                loop {
                    let n = stream.read(&mut buffer).await.map_err(|e| {
                        NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
                    })?;
                    if n == 0 {
                        break; // End of stream
                    }
                    response.extend_from_slice(&buffer[..n]);
                }

                let response: Self = bincode::deserialize(&response).map_err(|e| {
                    NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
                })?;
                Ok(response)
            }

        }
        
    };
}


#[macro_export]
macro_rules! register_contract_routes {
    ($handler_enum:ident, $fn_name:ident, $( $contract:ident => $handler_fn:path ),*) => {
        fn $fn_name(input_bytes: &[u8]) -> Result<Vec<u8>, NanoServiceError> {
            let received_msg: Result<$handler_enum, _> = bincode::deserialize(input_bytes);

            match received_msg {
                Ok(msg) => match msg {
                    $(
                        $handler_enum::$contract(inner) => {
                            let handled: $contract = $contract::handle(inner, $handler_fn)?;
                            Ok(bincode::serialize(&handled).map_err(|e| {
                                NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
                            })?)
                        }
                    )*
                    _ => Err(NanoServiceError::new(
                            "Received unknown contract type.".to_string(),
                            NanoServiceErrorStatus::BadRequest
                        )),
                },
                Err(e) => Err(NanoServiceError::new(
                        e.to_string(),
                        NanoServiceErrorStatus::BadRequest
                    )),
            }
        }
    };
}



#[cfg(test)]
mod tests {

    use super::*;
    use crate::networking::contract::Contract;
    use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
    use bitcode::{Decode, Encode};
    use serde::{Serialize, Deserialize};
    use tokio::net::TcpStream;
    use tokio::io::{AsyncWriteExt, AsyncReadExt};

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    pub struct TestContract {
        data: String,
        result: Option<String>,
        error: Option<NanoServiceError>
    }

    impl Contract<String, String> for TestContract {
        fn data(&self) -> Result<String, NanoServiceError> {
            Ok(self.data.clone())
        }

        fn result(&self) -> Result<String, NanoServiceError> {
            if let Some(result) = &self.result {
                Ok(result.clone())
            }
            else {
                Err(NanoServiceError::new(
                    "No result found.".to_string(),
                    NanoServiceErrorStatus::NotFound
                ))
            }
        }
    }

    #[derive(Debug, PartialEq, Encode, Decode, Deserialize, Serialize)]
    pub struct TestContractTwo {
        data: i32,
        result: i32
    }

    impl Contract<i32, i32> for TestContractTwo {
        fn data(&self) -> Result<i32, NanoServiceError> {
            Ok(self.data)
        }

        fn result(&self) -> Result<i32, NanoServiceError> {
            Ok(self.result)
        }
    }

    create_contract_handler!(
        TestContractHandler, 
        TestContract, 
        TestContractTwo
    );

    fn handle_test_contract(contract: TestContract) -> Result<TestContract, NanoServiceError> {
        Ok(contract)
    }

    register_contract_routes!(
        TestContractHandler, 
        handle_test_contracts, 
        TestContract => handle_test_contract
    );



    #[test]
    fn test_create_contract_handler() {
        let contract = TestContract {
            data: "Hello".to_string(),
            result: Some("World".to_string()),
            error: None
        };

        let handler = TestContractHandler::TestContract(contract);
        println!("{:?}", handler);
        let outcome = handler.TestContract().unwrap();
        println!("{:?}", outcome);
        // assert_eq!(handler, TestContractHandler::TestContract(contract));
    }

    #[test]
    fn test_register_contract_routes() {
        let contract = TestContractHandler::TestContract(TestContract {
            data: "Hello".to_string(),
            result: Some("World".to_string()),
            error: None
        });

        let encoded = bincode::serialize(&contract).unwrap();
        // assert_eq!(decoded, contract);

        let handled = handle_test_contracts(&encoded).unwrap();
        let decoded: TestContract = bincode::deserialize(&handled).unwrap();
        println!("{:?}", decoded.result());

        let false_contract = TestContractHandler::TestContractTwo(TestContractTwo {
            data: 1,
            result: 2,
        });

        let encoded = bincode::serialize(&false_contract).unwrap();
        let handled = handle_test_contracts(&encoded);
        println!("{:?}", handled);
        // assert_eq!(decoded, contract);
    }

}