use paho_mqtt as mqtt;
use std::error::Error;
use std::thread;

use std::time::Duration;
mod variables;
use futures::Future;

const HOST: &str = "test.mosquitto.org:1883";
const SUBSCRIBED_TOPICS: &[&str] = &["test", "ping-ask"];
const QUALITIES_OF_SERVICE: &[i32; 2] = &[2, 2];

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating an asynchronous mqtt client...");
    let mut client = new_client(HOST)?;

    client.set_connection_lost_callback(initiate_reconnection);

    client.set_message_callback(handle_messages);

    println!("connecting to the broker {}", HOST);
    let connect_options = create_connecting_options();

    client.connect_with_callbacks(connect_options, connect_success_cb, connect_failure_cb);

    // wait for incoming messages
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}

fn new_client(host: &str) -> mqtt::errors::MqttResult<mqtt::AsyncClient> {
    let create_options = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id("async_pong")
        .persistence(mqtt::PersistenceType::None)
        .finalize();

    mqtt::AsyncClient::new(create_options)
}

fn create_connecting_options() -> mqtt::ConnectOptions {
    let last_will_and_testament = mqtt::MessageBuilder::new()
        .topic("pong-response")
        .payload("the asynchronized ponger lost connection")
        .finalize();

    let connect_options = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        .will_message(last_will_and_testament)
        .clean_session(true)
        .finalize();
    connect_options
}

//             _ _ _                _
//    ___ __ _| | | |__   __ _  ___| | _____
//   / __/ _` | | | '_ \ / _` |/ __| |/ / __|
//  | (_| (_| | | | |_) | (_| | (__|   <\__ \
//   \___\__,_|_|_|_.__/ \__,_|\___|_|\_\___/

fn initiate_reconnection(client: &mqtt::AsyncClient) {
    println!("Connection lost. Attempting to reconnect");
    thread::sleep(Duration::from_millis(2500));
    client.reconnect_with_callbacks(connect_success_cb, connect_failure_cb);
}

fn connect_success_cb(client: &mqtt::AsyncClient, _message_id: u16) {
    println!("Connection succeeded");
    // subscribe to the desired topics
    client.subscribe_many(SUBSCRIBED_TOPICS, QUALITIES_OF_SERVICE);
    println!("Subscribing to topics: {:?}", SUBSCRIBED_TOPICS);
}

// this is a sleep & retry that calls itself recursively
fn connect_failure_cb(client: &mqtt::AsyncClient, _message_id: u16, return_code: i32) {
    println!(
        "Connection attempt failed with return code {}.\n",
        return_code
    );
    thread::sleep(Duration::from_millis(2500));
    client.reconnect_with_callbacks(connect_success_cb, connect_failure_cb);
}

fn handle_messages(client: &mqtt::AsyncClient, wrapped_message: Option<mqtt::Message>) {
    match wrapped_message {
        Some(message) => {
            // let topic = message.topic();
            // the payload is of type &[u8] so...
            let payload_string: &str = match std::str::from_utf8(message.payload()) {
                Ok(str) => str,
                Err(error) => panic!("Couldn't unpack the message payload: {}", error),
            };
            match payload_string {
                "ping" => {
                    println!("{}\nWe received a ping! Let's pong back!", payload_string);
                    publish_pong(client, "pong-response", "pong").unwrap();
                }
                _ => println!("{}\nThat wasn't a ping...", payload_string),
            }
        }
        None => println!("Well.... nothing"),
    }
}

fn publish_pong(
    client: &mqtt::AsyncClient,
    topic: &str,
    message: &str,
) -> mqtt::errors::MqttResult<()> {
    let ping_message = mqtt::MessageBuilder::new()
        .topic(topic)
        .payload(message)
        .qos(2)
        .finalize();

    println!("ponging");
    client.publish(ping_message).wait()
}
