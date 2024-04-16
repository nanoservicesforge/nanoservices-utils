use crate::errors::NanoServiceError;
use serde::{Serialize, de::DeserializeOwned};


pub trait Contract<D, R>: Sized + Serialize + DeserializeOwned {

    fn handle(
        message: Self, 
        handle_fn: fn(Self) -> Result<Self, NanoServiceError>
    ) -> Result<Self, NanoServiceError> {
        return handle_fn(message)
    }
    
    fn data(&self) -> Result<D, NanoServiceError>;
    
    fn result(&self) -> Result<R, NanoServiceError>;

}


pub trait ContractHandler {

    fn handle_contract(&self, contract: Vec<u8>) -> Result<Vec<u8>, NanoServiceError>;

    // fn package_contract<T: Contract>(contract: T) -> Result<Self, NanoServiceError>;

}


#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestContract {
        data: String,
        result: String
    }

    impl Contract<String, String> for TestContract {
        fn data(&self) -> Result<String, NanoServiceError> {
            Ok(self.data.clone())
        }

        fn result(&self) -> Result<String, NanoServiceError> {
            Ok(self.result.clone())
        }
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestContractTwo {
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


    #[tokio::test]
    async fn test_handle_contract() {
        let contract = TestContract {
            data: "Hello".to_string(),
            result: "World".to_string()
        };

        let result = TestContract::handle(contract, |c| {
            Ok(c)
        }).unwrap();

        assert_eq!(&result.data().unwrap(), &"Hello".to_string());
        assert_eq!(&result.result().unwrap(), &"World".to_string());
        // assert_eq!(result.unwrap().result(), Ok("World".to_string()));
    }
}