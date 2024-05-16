//! Wrapper are a serialization approach that allows contracts to be sent over TCP with custom serialization formats
//! as they serialize the contract, wrap themselves around the contract with data about the contract bytes, and then
//! send/recieve the contract over TCP. This removes the need for Tokio framing which also enables blocking
//! TCP calls.
pub mod bincode;
pub mod bitcode;
