use std::error::Error;
mod async_lib;
use async_lib::{AsyncMqttClient, AsyncMqttClientBuilder};
mod variables;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating an asynchronous mqtt client...");
    let async_mqtt_client = AsyncMqttClientBuilder::new_with_defaults()
        .with_host("test.mosquitto.org:1883")
        .with_client_id("async_ping")
        .with_publishing_topic("ping-ask")
        .with_subscribed_topic("pong-response")
        .with_last_will_and_testament("the synchronised pinger lost the connection")
        .with_quality_of_service(2)
        .with_clean_session(true)
        .build()?;

    async_mqtt_client.connect()?;
    async_mqtt_client.subscribe()?;

    async_mqtt_client.publish("ping")?;

    // wait for incoming messages
    loop {
        async_mqtt_client.reconnect_if_needed()?;

        if async_mqtt_client.received("pong")? {
            println!("Hurray my friend ponged back!");
            // todo: implement a break
        }
        thread::sleep(Duration::from_millis(200));
    }
}
