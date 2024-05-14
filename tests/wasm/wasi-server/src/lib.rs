use nanoservices_utils::errors::{NanoServiceError, NanoServiceErrorStatus};
use kernel::{
    ContractHandler,
    ContractOne,
    ContractTwo,
};
use nanoservices_utils::register_wasm_contract_routes;
use paste::paste;


fn handle_contract_one(mut contract: ContractOne) -> Result<ContractOne, NanoServiceError> {
    contract.name = "Bob".to_string();
    Ok(contract)
}

fn handle_contract_two(contract: ContractTwo) -> Result<ContractTwo, NanoServiceError> {
    Ok(contract)
}


register_wasm_contract_routes!(
    ContractHandler,
    handle_contract_routes,
    ContractOne => handle_contract_one,
    ContractTwo => handle_contract_two
);
