use crate::errors::NanoServiceError;
use serde::{Serialize, de::DeserializeOwned, Deserialize};


pub trait Contract<D, R>: Sized + Serialize + DeserializeOwned {
    
    fn check_data(&mut self) -> bool;
    
    fn check_result(&mut self) -> bool;

}


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BiDirectionalContract<D, R> {
    pub data: Option<D>,
    pub result: Option<R>,
    pub error: Option<NanoServiceError>
}

impl <D: Sized + Serialize + DeserializeOwned, R: Sized + Serialize + DeserializeOwned>Contract<D, R> for BiDirectionalContract<D, R> {
    
    fn check_data(&mut self) -> bool {
        if let Some(_) = &self.data {
            true
        }
        else {
            self.error = Some(NanoServiceError::new(
                "No data found.".to_string(),
                crate::errors::NanoServiceErrorStatus::BadRequest
            ));
            false
        }
    }

    fn check_result(&mut self) -> bool {
        if let Some(_) = &self.result {
            true
        }
        else {
            self.error = Some(NanoServiceError::new(
                "No result found.".to_string(),
                crate::errors::NanoServiceErrorStatus::Unknown
            ));
            false
        }
    }

}


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use serde::Deserialize;

//     #[derive(Debug, PartialEq, Serialize, Deserialize)]
//     struct TestContract {
//         data: String,
//         result: String
//     }

//     impl Contract<String, String> for TestContract {
//         fn data(&self) -> Result<String, NanoServiceError> {
//             Ok(self.data.clone())
//         }

//         fn result(&self) -> Result<String, NanoServiceError> {
//             Ok(self.result.clone())
//         }
//     }

//     #[derive(Debug, PartialEq, Serialize, Deserialize)]
//     struct TestContractTwo {
//         data: i32,
//         result: i32
//     }

//     impl Contract<i32, i32> for TestContractTwo {
//         fn data(&self) -> Result<i32, NanoServiceError> {
//             Ok(self.data)
//         }

//         fn result(&self) -> Result<i32, NanoServiceError> {
//             Ok(self.result)
//         }
//     }


//     #[tokio::test]
//     async fn test_handle_contract() {
//         let contract = TestContract {
//             data: "Hello".to_string(),
//             result: "World".to_string()
//         };

//         // let result = TestContract::handle(contract, |c| {
//         //     Ok(c)
//         // }).unwrap();

//         // assert_eq!(&result.data().unwrap(), &"Hello".to_string());
//         // assert_eq!(&result.result().unwrap(), &"World".to_string());
//         // assert_eq!(result.unwrap().result(), Ok("World".to_string()));
//     }
// }