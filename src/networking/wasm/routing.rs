

#[macro_export]
macro_rules! register_wasm_contract_routes {
    ($handler_enum:ident, $fn_name:ident, $( $contract:ident => $handler_fn:path ),*) => {
        fn $fn_name(received_msg: $handler_enum) -> Result<$handler_enum, NanoServiceError> {
            match received_msg {
                msg => match msg {
                    $(
                        $handler_enum::$contract(inner) => {
                            // need to add error handling
                            let executed_contract = $handler_fn(inner)?;
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

        extern crate alloc;
        use core::alloc::Layout;

        // for allocating memory
        #[no_mangle]
        pub unsafe extern "C" fn ns_malloc(size: u32, alignment: u32) -> *mut u8 {
            let layout = Layout::from_size_align_unchecked(size as usize, alignment as usize);
            alloc::alloc::alloc(layout)
        }

        // for deallocating memory
        #[no_mangle]
        pub unsafe extern "C" fn ns_free(ptr: *mut u8, size: u32, alignment: u32) {
            let layout = Layout::from_size_align_unchecked(size as usize, alignment as usize);
            alloc::alloc::dealloc(ptr, layout);
        }

        /// The pointer struct to be returned to the host machine.
        /// 
        /// # Fields
        /// - `ptr` - The pointer to the serialized data memory address
        /// - `len` - The length of the serialized data
        #[repr(C)]
        pub struct ContractPointer {
            ptr: i32,
            len: i32
        }

        $(
            paste! {
                #[no_mangle]
                pub extern "C" fn [<$contract:lower _contract>](ptr: *const u8, len: usize) -> *const ContractPointer {
                    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
                    let contract: $contract = bincode::deserialize(bytes).unwrap();
                    let result = $handler_fn(contract).unwrap();

                    let serialized_data = bincode::serialize(&result).unwrap();
                    let len = serialized_data.len();
                    let out_ptr = serialized_data.leak().as_ptr();

                    let result = Box::new(ContractPointer{
                        ptr: out_ptr as i32,
                        len: len as i32
                    });
                    Box::into_raw(result) as *const ContractPointer
                }
            }
        )*
    };
}
