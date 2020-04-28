use futures::Future;
use paho_mqtt as mqtt;
mod variables;
use std::thread;
use std::time::Duration;

fn main() {
    
    let host: String = variables::HOST.to_string();

    let create_options = mqtt::CreateOptionsBuilder::new()
        .server_uri(&host)
        // .client_id("sync_ping")
        .persistence(mqtt::PersistenceType::None)
        .finalize();

    let mut client = match mqtt::AsyncClient::new(create_options) {
        Ok(client) => client,
        Err(error) => panic!("error creating the client: {:?}", error),
    };

    client
        .connect(None)
        .and_then(|_| {
            println!("Pinging on topic 'ping-ask'");
            let ping_message = mqtt::Message::new("ping-ask", "ping", 2);
            client.publish(ping_message)
        })
        .and_then(|_| client.disconnect(None))
        .wait()
        .unwrap();
}
