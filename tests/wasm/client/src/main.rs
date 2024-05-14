use wasmtime::{Result, Engine, Linker, Module, Store, Config};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;
use std::mem::size_of;
use std::slice::from_raw_parts;
use nanoservices_utils::errors::{NanoServiceError, NanoServiceErrorStatus};
use kernel::{
    ContractHandler,
    ContractOne,
    ContractTwo,
};

#[repr(C)]
pub struct ContractPointer {
    ptr: i32,
    len: i32
}


// An example of executing a WASIp1 "command"
#[tokio::main]
async fn main() -> Result<()> {
    let mut config = Config::new();
    config.async_support(true);
    let engine = Engine::new(&config).unwrap();
    let module = Module::from_file(&engine, "../wasi-server/wasi_server.wasm").unwrap();

    let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
    preview1::add_to_linker_async(&mut linker, |t| t).unwrap();
    let pre = linker.instantiate_pre(&module)?;

    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_env()
        .build_p1();

    let mut store = Store::new(&engine, wasi_ctx);
    let instance = pre.instantiate_async(&mut store).await.unwrap();

    // put the stuff below as a loop in the actor 

    let contract = ContractHandler::ContractOne(ContractOne {
        name: "Alice".to_string(),
        age: 42,
    });
    let name_ref = contract.to_string_ref();
    
    let serialized = contract.to_contract_bytes().unwrap();

    // allocate the memory for the input data
    let malloc = instance.get_typed_func::<(i32, i32), i32>(&mut store, "ns_malloc").unwrap();
    let input_data_ptr = malloc.call_async(&mut store, (serialized.len() as i32, 0)).await.unwrap();

    // write the contract to the memory
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    memory.write(&mut store, input_data_ptr as usize, &serialized).unwrap();

    // load and call the entry point
    let entry_point = instance.get_typed_func::<(i32, i32), i32>(&mut store, &name_ref).unwrap();
    let ret = entry_point.call_async(&mut store, (input_data_ptr, serialized.len() as i32)).await.unwrap();

    let mut contract_result_buffer = Vec::with_capacity(size_of::<ContractPointer>());
    for _ in 0..size_of::<ContractPointer>() {
        contract_result_buffer.push(0);
    }
    memory.read(&mut store, ret as usize, &mut contract_result_buffer).unwrap();
    let result_struct = unsafe {
        &from_raw_parts::<ContractPointer>(contract_result_buffer.as_ptr() as *const ContractPointer, 1)[0]
    };

    let mut output_contract_buffer: Vec<u8> = Vec::with_capacity(result_struct.len as usize);
    output_contract_buffer.resize(result_struct.len as usize, 0);

    memory.read(&mut store, result_struct.ptr as usize, &mut output_contract_buffer).unwrap();
    let contract = ContractHandler::from_contract_bytes(&output_contract_buffer, name_ref).unwrap();
    println!("Output contract: {:?}", contract);

    let free = instance.get_typed_func::<(i32, i32, i32), ()>(&mut store, "ns_free").unwrap();
    free.call_async(&mut store, (input_data_ptr, serialized.len() as i32, 0)).await.unwrap();
    free.call_async(&mut store, (result_struct.ptr, result_struct.len, 0)).await.unwrap();
    free.call_async(&mut store, (ret, size_of::<ContractPointer>() as i32, 0)).await.unwrap();
    Ok(())
}


