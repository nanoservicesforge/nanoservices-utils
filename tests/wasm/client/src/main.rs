use std::process::{Command, Stdio};
use nanoservices_utils::errors::{NanoServiceError, NanoServiceErrorStatus};
use std::io::{self, BufRead, BufReader, Write};
use kernel::{
    ContractHandler,
    ContractOne,
    ContractTwo,
};


fn main() -> io::Result<()> {
    // Start the child process
    let mut child = Command::new("wasmtime")
        .arg("../wasi-server/wasi-server.wasm")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start WASM process");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let stdout = child.stdout.as_mut().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // Prepare and send multiple messages
    let messages = vec![
        ContractHandler::ContractOne(ContractOne {
            name: "Alice".to_string(),
            age: 42,
        }),
        ContractHandler::ContractTwo(ContractTwo {
            account_name: "Sample".to_string(),
            amount: 99,
        }),
    ];

    for message in messages {
        // message type 1 is the contract, will add things like type 2 for data storage later
        let message_type: u32 = 1;
        let message_type_bytes = message_type.to_be_bytes();
        let input_data = bincode::serialize(&message).unwrap();

        // pack with message type
        let mut encoded_message: Vec<u8> = Vec::with_capacity(message_type_bytes.len() + input_data.len());
        encoded_message.extend_from_slice(&message_type_bytes);
        encoded_message.extend_from_slice(&input_data);

        // Send the message to the child process
        stdin.write_all(&encoded_message)?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;

        // Read the response for each message
        let mut output = Vec::new();
        reader.read_until(b'\n', &mut output)?;

        let (type_prefix, message_data) = output.split_at(4);
        let request_type = u32::from_be_bytes([type_prefix[0], type_prefix[1], type_prefix[2], type_prefix[3]]);

        let result: ContractHandler = bincode::deserialize(&message_data).unwrap();
        println!("Received: {:?}", result);
    }

    Ok(())
}