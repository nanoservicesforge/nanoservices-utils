//! Handles the binding of multiple contracts to a handler.
//! 
//! # Example
//! 
//! ```rust
//! use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
//! use serde::{Serialize, Deserialize};
//! 
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! pub struct ContractOne;
//! 
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! pub struct ContractTwo;
//! 
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! pub struct ContractThree;
//! 
//! create_contract_handler!(
//!    ContractHandler,
//!    ContractOne,
//!    ContractTwo,
//!    ContractThree
//! );
//! 
//! // wrap a contract
//! let contract_one = ContractHandler::ContractOne(ContractOne);
//! 
//! // serialise a contract
//! let serialized_contract_one = bincode::serialize(&contract_one).unwrap();
//! 
//! // deserialise a contract
//! let deserialized_contract_one: ContractHandler = bincode::deserialize(&serialized_contract_one).unwrap();
//! 
//! // assert that the contracts are equal
//! assert_eq!(contract_one, deserialized_contract_one);
//! ```
//! This enables you to pass one of multiple contracts from one handler to another over a network. A `NanoserviceError` is
//! also attached to the handler so errors raw errors can be passed around as well.
#[macro_export]
macro_rules! create_contract_handler {
    ($enum_name:ident, $( $variant:ident ),*) => {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub enum $enum_name {
            $( $variant($variant), )+
            NanoServiceError(NanoServiceError)
        }

        impl $enum_name {
            $(
                #[allow(non_snake_case)]
                pub fn $variant(self) -> Result<$variant, NanoServiceError> {
                    match self {
                        $enum_name::$variant(inner) => Ok(inner),
                        $enum_name::NanoServiceError(inner) => Err(inner),
                        _ => Err(NanoServiceError::new(
                                format!("Expected variant: {}", stringify!($variant)),
                                NanoServiceErrorStatus::BadRequest
                            )
                        ),
                    }
                }
            )+

            #[allow(non_snake_case)]
            pub fn NanoServiceError(self) -> Result<NanoServiceError, NanoServiceError> {
                match self {
                    $enum_name::NanoServiceError(inner) => Ok(inner),
                    _ => Err(NanoServiceError::new(
                            "Expected variant: NanoServiceError".to_string(),
                            NanoServiceErrorStatus::BadRequest
                        )
                    ),
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {

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

    #[test]
    fn test_contract_handler_variants() {
        let contract_one = ContractHandler::ContractOne(ContractOne);
        let contract_two = ContractHandler::ContractTwo(ContractTwo);
        let contract_three = ContractHandler::ContractThree(ContractThree);

        assert_eq!(contract_one.ContractOne().unwrap(), ContractOne);
        assert_eq!(contract_two.ContractTwo().unwrap(), ContractTwo);
        assert_eq!(contract_three.ContractThree().unwrap(), ContractThree);
    }

    #[test]
    fn test_contract_serialization() {
        // define the contracts
        let contract_one = ContractHandler::ContractOne(ContractOne);
        let contract_two = ContractHandler::ContractTwo(ContractTwo);
        let contract_three = ContractHandler::ContractThree(ContractThree);
        let nanoservice_error = ContractHandler::NanoServiceError(NanoServiceError::new(
            "Test error".to_string(),
            NanoServiceErrorStatus::BadRequest
        ));

        // serialize the contracts
        let serialized_contract_one = bincode::serialize(&contract_one).unwrap();
        let serialized_contract_two = bincode::serialize(&contract_two).unwrap();
        let serialized_contract_three = bincode::serialize(&contract_three).unwrap();
        let serialized_nanoservice_error = bincode::serialize(&nanoservice_error).unwrap();

        // deserialize the contracts
        let deserialized_contract_one: ContractHandler = bincode::deserialize(&serialized_contract_one).unwrap();
        let deserialized_contract_two: ContractHandler = bincode::deserialize(&serialized_contract_two).unwrap();
        let deserialized_contract_three: ContractHandler = bincode::deserialize(&serialized_contract_three).unwrap();
        let deserialized_nanoservice_error: ContractHandler = bincode::deserialize(&serialized_nanoservice_error).unwrap();

        // assert that the contracts are equal
        assert_eq!(contract_one, deserialized_contract_one);
        assert_eq!(contract_two, deserialized_contract_two);
        assert_eq!(contract_three, deserialized_contract_three);
        assert_eq!(nanoservice_error, deserialized_nanoservice_error);
    }

    #[test]
    fn test_error_parsing() {
        let contract = ContractHandler::ContractOne(ContractOne);
        let error = ContractHandler::NanoServiceError(NanoServiceError::new(
            "Test error".to_string(),
            NanoServiceErrorStatus::BadRequest
        ));

        assert_eq!(contract.ContractOne().unwrap(), ContractOne);
        assert_eq!(error.NanoServiceError().unwrap().status, NanoServiceErrorStatus::BadRequest);
    }

    #[test]
    fn test_error_parsing_failure() {
        let contract = ContractHandler::ContractOne(ContractOne);
        let error = ContractHandler::NanoServiceError(NanoServiceError::new(
            "Test error".to_string(),
            NanoServiceErrorStatus::BadRequest
        ));

        // below we are trying to parse contract two when it contains contract one
        assert_eq!(contract.ContractTwo().unwrap_err().status, NanoServiceErrorStatus::BadRequest);
        assert_eq!(error.NanoServiceError().unwrap().status, NanoServiceErrorStatus::BadRequest);
    }

}