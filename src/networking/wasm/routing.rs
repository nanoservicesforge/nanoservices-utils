

#[macro_export]
macro_rules! register_contract_routes {
    ($handler_enum:ident, $fn_name:ident, $( $contract:ident => $handler_fn:path ),*) => {
        async fn $fn_name(received_msg: $handler_enum) -> Result<$handler_enum, NanoServiceError> {
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
        #[tokio::main(flavor="current_thread")]
        async fn main() -> io::Result<()> {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut reader = BufReader::new(stdin.lock());

            let mut line = String::new();
            let mut writer = stdout.lock();

            while reader.read_line(&mut line)? != 0 {
                let input_bytes = line.trim().as_bytes();
                let (type_prefix, message_data) = input_bytes.split_at(4);
                let request_type = u32::from_be_bytes([type_prefix[0], type_prefix[1], type_prefix[2], type_prefix[3]]);

                match request_type {
                    1 => {
                        // handle contract routes
                        let request: ContractHandler = bincode::deserialize(message_data).unwrap();
                        let response = $fn_name(request).await.unwrap();

                        // serialise data
                        let encoded_response = bincode::serialize(&response).unwrap();
                        let message_type: u32 = 1;
                        let message_type_bytes = message_type.to_be_bytes();

                        // pack data
                        let mut encoded_message: Vec<u8> = Vec::with_capacity(message_type_bytes.len() + encoded_response.len());
                        encoded_message.extend_from_slice(&message_type_bytes);
                        encoded_message.extend_from_slice(&encoded_response);

                        // send data
                        writer.write_all(&encoded_message)?;
                        writer.write_all(b"\n")?;
                        writer.flush()?;
                        line.clear();
                    },
                    // right now it will just crash but should not have anything here
                    _ => println!("Received unknown request type"),
                }
            }
            Ok(())
        }

        // expose the wasm enrtypoint
        // #[wasm_bindgen]
        // #[no_mangle]
        // pub async fn entry_point(ptr: *const u8, len: usize) -> *const u8  {
        //     let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        //     // Deserialize the bytes into the handler enum
        //     let handler_enum: $handler_enum = bincode::deserialize(bytes).unwrap();

        //     // Call the async function
        //     let result = $fn_name(handler_enum).await;

        //     // Serialize the enum to bytes
        //     let serialized_data = bincode::serialize(&result).unwrap();
        //     let leaked_data = Box::leak(serialized_data.into_boxed_slice());
        //     leaked_data.as_ptr()
        // }

    };
}
