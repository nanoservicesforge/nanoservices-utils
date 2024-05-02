use nanoservices_utils::create_contract_handler;
use nanoservices_utils::errors::{NanoServiceError, NanoServiceErrorStatus};
use serde::{Deserialize, Serialize};


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractOne {
    pub name: String,
    pub age: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractTwo {
    pub account_name: String,
    pub amount: u32,
}

create_contract_handler!(
    ContractHandler, 
    ContractOne, 
    ContractTwo
);
