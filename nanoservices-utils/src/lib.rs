//! This crate is a basic utils crate that helps glue nanoservices together.
#[allow(dead_code)]
pub mod errors;

#[cfg(feature = "jwt")]
#[allow(dead_code)]
pub mod jwt;
#[allow(dead_code)]
pub mod config;

#[cfg(feature = "networking")]
#[allow(dead_code)]
pub mod networking;


#[cfg(feature = "dal")]
#[allow(dead_code)]
pub mod data_access;


#[cfg(feature = "dal")]
#[allow(dead_code)]
pub use nan_serve_dal_tx_impl::impl_transaction;

#[cfg(feature = "tokio-pub-sub")]
#[allow(dead_code)]
pub mod tokio_pub_sub;

#[cfg(feature = "tokio-pub-sub")]
#[allow(dead_code)]
pub use ctor;

#[cfg(feature = "tokio-pub-sub")]
#[allow(dead_code)]
pub use bincode;

#[cfg(feature = "tokio-pub-sub")]
#[allow(dead_code)]
pub use nan_serve_event_subscriber::subscribe_to_event;

#[cfg(feature = "tokio-pub-sub")]
#[allow(dead_code)]
pub use nan_serve_publish_event::publish_event;
