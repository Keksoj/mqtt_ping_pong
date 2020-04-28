use futures::Future;
use paho_mqtt as mqtt;
use std::error::Error;
mod async_lib;
use async_lib::{AsyncMqttClient, AsyncMqttClientBuilder};
mod variables;
use std::thread;
use std::time::Duration;

const HOST: &str = "test.mosquitto.org:1883";
const QUALITY_OF_SERVICE: i32 = 2;
const TOPIC: &str = "pong-response";

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating an asynchronous mqtt client...");
    let mut client = AsyncMqttClientBuilder::new_with_defaults()
        .with_host("test.mosquitto.org:1883")
        .with_client_id("async_ping")
        .with_publishing_topic("ping-ask")
        .with_subscribed_topic("pong-response")
        .with_last_will_and_testament("the synchronised ponger lost the connection")
        .with_quality_of_service(2)
        .with_clean_session(true)
        .build()?;

    client
        .client
        .set_connection_lost_callback(initiate_reconnection);

    client.client.set_message_callback(handle_messages);

    println!("connecting to the broker {}", HOST);
    let connect_options = create_connecting_options(&TOPIC);
    client
        .client
        .connect_with_callbacks(connect_options, connect_success_cb, connect_failure_cb)
        .wait()?;

    publish_ping(&client.client, "ping-ask", "ping")?;

    // wait for incoming messages
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}


fn create_connecting_options(topic: &str) -> mqtt::ConnectOptions {
    let last_will_and_testament = mqtt::MessageBuilder::new()
        .topic(topic)
        .payload("the asynchronized pinger lost connection")
        .finalize();

    let connect_options = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        .will_message(last_will_and_testament)
        .clean_session(true)
        .finalize();
    connect_options
}

fn publish_ping(
    client: &mqtt::AsyncClient,
    topic: &str,
    message: &str,
) -> mqtt::errors::MqttResult<()> {
    let ping_message = mqtt::MessageBuilder::new()
        .topic(topic)
        .payload(message)
        .qos(2)
        .finalize();

    println!("Sending the ping");
    client.publish(ping_message).wait()
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
    client.subscribe(TOPIC, QUALITY_OF_SERVICE);
    println!("Subscribing to topics: {:?}", TOPIC);
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

fn handle_messages(_client: &mqtt::AsyncClient, wrapped_message: Option<mqtt::Message>) {
    let message = wrapped_message.unwrap();
    let payload_string: &str = std::str::from_utf8(message.payload()).unwrap();
    println!("{}", payload_string);
    match payload_string {
        "pong" => println!("We received the pong!"),
        _ => println!("That wasn't a pong..."),
    }
}
