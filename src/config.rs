//! Defines extracting config variables based on the method to enable testing as well as production use.
//! A good is API views. Wehn when defining an API view function, we can use the `GetConfigVariable` trait to
//! to enable config variables from different sources be applied to the API view function by having the following
//! outline:
//! 
//! ```rust
//! use nanoservices_utils::config::GetConfigVariable;
//! 
//! pub async fn api_view<T: GetConfigVariable>() {
//! }
//! ```
//! Anything passed into the `api_view` function will be able to get config variables from the source defined by
//! the type of the generic parameter `T`. For example, if we want to get config variables from the environment, we
//! can pass the `EnvConfig` struct as `api_view::<EnvConfig>()`. However, we can implement the `GetConfigVariable`
//! on a random struct for unit testing and pass that struct into the `api_view` function for testing.
//! use nanoservices_utils::config::GetConfigVariable;
//! 
//! let _ = 
use std::env;
use crate::errors::{
    NanoServiceError,
    NanoServiceErrorStatus
};


/// Used for extracting config cariables.
pub trait GetConfigVariable {

    /// Gets the config variable
    ///
    /// # Arguments
    /// * `variable` - The name of the config variable to get
    ///
    /// # Returns
    /// * `Result<String, NanoServiceError>` - The result of getting the config variable
    fn get_config_variable(variable: String) -> Result<String, NanoServiceError>;
}


/// Defines the struct for getting config variables from the environment
pub struct EnvConfig;

impl GetConfigVariable for EnvConfig {

    /// Gets the config variable from the environment
    ///
    /// # Arguments
    /// * `variable` - The name of the config variable to get
    ///
    /// # Returns
    /// * `Result<String, NanoServiceError>` - The result of getting the config variable
    fn get_config_variable(variable: String) -> Result<String, NanoServiceError> {
        match env::var(&variable) {
            Ok(val) => Ok(val),
            Err(_) => Err(
                NanoServiceError::new(
                    format!("{} not found in environment", variable),
                    NanoServiceErrorStatus::Unknown
                )
            )
        }
    }
}
