use nanoservices_utils::networking::tcp::wasm_proxy::TcpToWasmProxy;
use kernel::{
    ContractHandler,
    ContractOne,
    ContractTwo,
};

// "../wasi-server/wasi-server.wasm"


#[tokio::main]
async fn main() {
    let proxy = TcpToWasmProxy::new(
        "127.0.0.0:8001".to_string(),
        "../wasi-server/wasi-server.wasm".to_string(),
    );
    let outcome = proxy.start::<ContractHandler>().await.unwrap();
}
