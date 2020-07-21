use paho_mqtt as mqtt;
use std::error::Error;
use std::thread;

mod async_lib;
use async_lib::{AsyncMqttClient, AsyncMqttClientBuilder};
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating an asynchronous mqtt client...");
    let async_mqtt_client = AsyncMqttClientBuilder::new_with_defaults()
        .with_host("test.mosquitto.org:1883")
        .with_client_id("Synchronised ponger")
        .with_publishing_topic("pong-response")
        .with_subscribed_topic("ping-ask")
        .with_last_will_and_testament(
            "the synchronised ponger lost the connection",
        )
        .with_quality_of_service(2)
        .with_clean_session(true)
        .build()?;

    async_mqtt_client.connect()?;
    async_mqtt_client.subscribe()?;

    // this would be rad:
    async_mqtt_client.set_message_callback(user_written_callback);

    loop {
        async_mqtt_client.reconnect_if_needed()?;
        if async_mqtt_client.received("ping")? {
            println!("We received a ping! Let's pong back");
            async_mqtt_client.publish("pong")?;
            println!("My job here is done");

            // todo: implement a break
        }
        thread::sleep(Duration::from_millis(200));
    }
}

// What I'd like would be to enable this
fn user_written_callback(
    async_mqtt_client: &AsyncMqttClient,
    message: Option<mqtt::Message>,
) -> Result<(), Box<dyn Error>> {
    let message = message.unwrap();
    let content: &str = std::str::from_utf8(message.payload())?;
    println!("That message is lit: {}", content);
    println!("let's republish it.");
    async_mqtt_client.publish(content)?;
    Ok(())
}
