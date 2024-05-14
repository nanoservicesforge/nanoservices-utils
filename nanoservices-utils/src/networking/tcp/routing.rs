#[macro_export]
macro_rules! register_contract_routes {
    ($handler_enum:ident, $fn_name:ident, $( $contract:ident => $handler_fn:path ),*) => {
        pub async fn $fn_name(received_msg: $handler_enum) -> Result<$handler_enum, NanoServiceError> {
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
                            NanoServiceErrorStatus::ContractNotSupported
                        )),
                },
            }
        }
    };
}


#[cfg(test)]
mod tests {

    use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
    use serde::{Serialize, Deserialize};
    use crate::create_contract_handler;
    use tokio::runtime::Builder;


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

    #[test]
    fn test_register_contract_routes() {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let contract_one = ContractHandler::ContractOne(ContractOne);
            let contract_two = ContractHandler::ContractTwo(ContractTwo);
            let contract_three = ContractHandler::ContractThree(ContractThree);

            let handled_contract_one = handle_contract(contract_one).await.unwrap();
            let handled_contract_two = handle_contract(contract_two).await.unwrap();
            let handled_contract_three = handle_contract(contract_three).await;

            assert_eq!(handled_contract_one, ContractHandler::ContractOne(ContractOne));
            assert_eq!(handled_contract_two, ContractHandler::ContractTwo(ContractTwo));
            assert_eq!(handled_contract_three, Err(NanoServiceError::new(
                "Received unknown contract type.".to_string(),
                NanoServiceErrorStatus::ContractNotSupported
            )));
        });
    }

}