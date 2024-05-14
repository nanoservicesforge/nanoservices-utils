//! Basic utils module that can be used in any networking related code.
use std::net::{TcpListener, SocketAddr};


/// Find an available port on the system.
/// 
/// # Returns
/// - `Some(u32)` - The available port number.
pub fn find_available_port() -> Option<u32> {
    (8000..65535).find_map(|port| {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        TcpListener::bind(addr).ok().map(|listener| {
            // Extract the port from the listener
            listener.local_addr().unwrap().port() as u32
        })
    })
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_find_available_port() {
        let port = find_available_port().unwrap();
        assert!(port >= 8000 && port <= 65535);
    }

}
