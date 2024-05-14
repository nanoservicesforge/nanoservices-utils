//! Handles the binding of multiple contracts to a handler.
//! 
//! # Example
//! 
//! ```rust
//! use nanoservices_utils::errors::{NanoServiceError, NanoServiceErrorStatus};
//! use nanoservices_utils::create_contract_handler;
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

            pub fn to_string_ref(&self) -> String {
                match self {
                    $(
                        $enum_name::$variant(_) => format!("{}_contract", stringify!($variant).to_lowercase()),
                    )+
                    $enum_name::NanoServiceError(_) => "nanoService_error".to_string(),
                }
            }

            pub fn from_contract_bytes(bytes: &[u8], string_ref: String) -> Result<$enum_name, NanoServiceError> {
                $(
                    if string_ref == format!("{}_contract", stringify!($variant).to_lowercase()) {
                        if let Ok(contract) = bincode::deserialize::<$variant>(bytes) {
                            return Ok($enum_name::$variant(contract));
                        }
                    }
                )+
                return Err(NanoServiceError::new(
                    "Failed to deserialize contract".to_string(),
                    NanoServiceErrorStatus::BadRequest
                ))
            }

            pub fn to_contract_bytes(&self) -> Result<Vec<u8>, NanoServiceError> {
                match self {
                    $(
                        $enum_name::$variant(contract) => {
                            if let Ok(bytes) = bincode::serialize(contract) {
                                return Ok(bytes)
                            }
                        }
                    )+
                    $enum_name::NanoServiceError(error) => {
                        if let Ok(bytes) = bincode::serialize(error) {
                            return Ok(bytes)
                        }
                    }
                }
                return Err(NanoServiceError::new(
                    "Failed to serialize contract".to_string(),
                    NanoServiceErrorStatus::BadRequest
                ))
            }

            pub fn internal_index(&self) -> i32 {
                let mut index = 0;
                $(
                    index += 1;
                    if let $enum_name::$variant(_) = self {
                        return index
                    }
                )+
                return 0
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
    fn test_from_contract_bytes() {
        let contract_one = ContractOne;
        let bytes = bincode::serialize(&contract_one).unwrap();
        let contract_handler = ContractHandler::from_contract_bytes(
            &bytes,
            "contractone_contract".to_string()
        ).unwrap();
        assert_eq!(contract_handler, ContractHandler::ContractOne(contract_one));

        let contract_two = ContractTwo;
        let bytes = bincode::serialize(&contract_two).unwrap();
        let contract_handler = ContractHandler::from_contract_bytes(
            &bytes,
            "contracttwo_contract".to_string()
        ).unwrap();
        assert_eq!(contract_handler, ContractHandler::ContractTwo(contract_two));

        let contract_three = ContractThree;
        let bytes = bincode::serialize(&contract_three).unwrap();
        let contract_handler = ContractHandler::from_contract_bytes(
            &bytes,
            "contractthree_contract".to_string()
        ).unwrap();
        assert_eq!(contract_handler, ContractHandler::ContractThree(contract_three));
    }

    #[test]
    fn test_to_contract_bytes() {
        let contract_one = ContractOne;
        let contract_one_ref = ContractOne;
        let contract_handler = ContractHandler::ContractOne(contract_one);
        let string_ref = contract_handler.to_string_ref();
        let bytes = contract_handler.to_contract_bytes().unwrap();
        let bytes_ref = bytes.clone();
        assert_eq!(contract_handler, ContractHandler::from_contract_bytes(
            &bytes,
            string_ref
        ).unwrap());
        let deserialized_contract: ContractOne = bincode::deserialize(&bytes_ref).unwrap();
        assert_eq!(contract_one_ref, deserialized_contract);

        let contract_two = ContractTwo;
        let contract_two_ref = ContractTwo;
        let contract_handler = ContractHandler::ContractTwo(contract_two);
        let string_ref: String = contract_handler.to_string_ref();
        let bytes = contract_handler.to_contract_bytes().unwrap();
        let bytes_ref = bytes.clone();
        assert_eq!(contract_handler, ContractHandler::from_contract_bytes(
            &bytes,
            string_ref
        ).unwrap());
        let deserialized_contract: ContractTwo = bincode::deserialize(&bytes_ref).unwrap();
        assert_eq!(contract_two_ref, deserialized_contract);

        let contract_three = ContractThree;
        let contract_three_ref = ContractThree;
        let contract_handler = ContractHandler::ContractThree(contract_three);
        let string_ref: String = contract_handler.to_string_ref();
        let bytes = contract_handler.to_contract_bytes().unwrap();
        let bytes_ref = bytes.clone();
        assert_eq!(contract_handler, ContractHandler::from_contract_bytes(
            &bytes,
            string_ref
        ).unwrap());
        let deserialized_contract: ContractThree = bincode::deserialize(&bytes_ref).unwrap();
        assert_eq!(contract_three_ref, deserialized_contract);
    }

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
    fn test_contract_handler_string_refs() {
        let contract_one = ContractHandler::ContractOne(ContractOne);
        let contract_two = ContractHandler::ContractTwo(ContractTwo);
        let contract_three = ContractHandler::ContractThree(ContractThree);
        let nanoservice_error = ContractHandler::NanoServiceError(NanoServiceError::new(
            "Test error".to_string(),
            NanoServiceErrorStatus::BadRequest
        ));

        assert_eq!(contract_one.to_string_ref(), "contractone_contract");
        assert_eq!(contract_two.to_string_ref(), "contracttwo_contract");
        assert_eq!(contract_three.to_string_ref(), "contractthree_contract");
        assert_eq!(nanoservice_error.to_string_ref(), "nanoService_error");
    }

    #[test]
    fn test_contract_indexes() {
        let contract_three = ContractHandler::ContractThree(ContractThree);
        let contract_one = ContractHandler::ContractOne(ContractOne);
        let contract_two = ContractHandler::ContractTwo(ContractTwo);
        let nanoservice_error = ContractHandler::NanoServiceError(NanoServiceError::new(
            "Test error".to_string(),
            NanoServiceErrorStatus::BadRequest
        ));

        assert_eq!(contract_one.internal_index(), 1);
        assert_eq!(contract_two.internal_index(), 2);
        assert_eq!(contract_three.internal_index(), 3);
        assert_eq!(nanoservice_error.internal_index(), 0);
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