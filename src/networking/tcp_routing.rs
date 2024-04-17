

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

            pub async fn send_over_tcp(self, address: &str) -> Result<Self, NanoServiceError> {
                let stream = TcpStream::connect(address).await.map_err(|e| {
                    NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::BadRequest)
                })?;
                let mut framed = Framed::new(stream, BincodeCodec::<Self>::new());
                framed.send(self).await.map_err(|e| {
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

        }
        
    };
}


#[macro_export]
macro_rules! register_contract_routes {
    ($handler_enum:ident, $fn_name:ident, $( $contract:ident => $handler_fn:path ),*) => {
        async fn $fn_name(received_msg: $handler_enum) -> Result<$handler_enum, NanoServiceError> {
            match received_msg {
                msg => match msg {
                    $(
                        $handler_enum::$contract(inner) => {
                            // need to add error handling
                            let executed_contract = $handler_fn(inner).await?;
                            return Ok($handler_enum::$contract(executed_contract));
                        }
                    )*
                    _ => Err(NanoServiceError::new(
                            "Received unknown contract type.".to_string(),
                            NanoServiceErrorStatus::BadRequest
                        )),
                },
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
    use tokio_util::codec::Framed;
    use crate::networking::codec::BincodeCodec;
    use futures::sink::SinkExt;
    use futures::StreamExt;

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

    async fn handle_test_contract(contract: TestContract) -> Result<TestContract, NanoServiceError> {
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
        // let outcome = handler.TestContract().unwrap();
        // println!("{:?}", outcome);
        // assert_eq!(handler, TestContractHandler::TestContract(contract));
    }

    #[test]
    fn test_register_contract_routes() {
        let contract = TestContractHandler::TestContract(TestContract {
            data: "Hello".to_string(),
            result: Some("World".to_string()),
            error: None
        });

        // // let encoded = bincode::serialize(&contract).unwrap();
        // // assert_eq!(decoded, contract);

        // let handled = handle_test_contracts(contract).unwrap();
        // // let decoded: TestContract = bincode::deserialize(&handled).unwrap();
        // println!("{:?}", handled.TestContract().unwrap().result());

        // let false_contract = TestContractHandler::TestContractTwo(TestContractTwo {
        //     data: 1,
        //     result: 2,
        // });

        // let encoded = bincode::serialize(&false_contract).unwrap();
        // let handled = handle_test_contracts(false_contract);
        // println!("{:?}", handled);
        // assert_eq!(decoded, contract);
    }

}