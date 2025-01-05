

#[macro_export]
macro_rules! config_tokio_event_runtime {
    () => {
        pub mod tokio_event_adapter_runtime {

            use std::sync::{Arc, RwLock, LazyLock};
            use std::collections::HashMap;
            use serde::{Serialize, Deserialize};
            use std::future::Future;
            use std::pin::Pin;

            pub type EventFunctionBuffer = Vec<EventFunction>;
            pub type EventFunction = fn(Vec<u8>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

            static HASHMAP: LazyLock<Arc<RwLock<HashMap<String, EventFunctionBuffer>>>> = LazyLock::new(|| {
                Arc::new(RwLock::new(HashMap::new()))
            });

            pub fn insert_into_hashmap(name: String, func: EventFunction) -> () {
                let mut buffer = get_from_hashmap(&name).unwrap_or_else(|| vec![]);
                buffer.push(func);
                let mut map = HASHMAP.write().unwrap();
                map.insert(name, buffer);
            }

            pub fn get_from_hashmap(name: &str) -> Option<EventFunctionBuffer> {
                HASHMAP.read().unwrap().get(name).cloned()
            }

            pub fn publish_event(name: &str, data: Vec<u8>) -> () {
                let buffer = match get_from_hashmap(name) {
                    Some(b) => b,
                    None => {
                        println!("No subscribers for event: {}", name);
                        return
                    }
                };
                for f in buffer {
                    let boxed_future = f(data.clone());
                    tokio::spawn(async move {
                        boxed_future.await;
                    });
                }
            }

        }
    };
}