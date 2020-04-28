use std::error::Error;
use std::thread;
mod async_lib;
use async_lib::AsyncMqttClientBuilder;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating an asynchronous mqtt client...");
    let async_mqtt_client = AsyncMqttClientBuilder::new_with_defaults()
        .with_host("test.mosquitto.org:1883")
        .with_client_id("Synchronised ponger")
        .with_publishing_topic("pong-response")
        .with_subscribed_topic("ping-ask")
        .with_last_will_and_testament("the synchronised ponger lost the connection")
        .with_quality_of_service(2)
        .with_clean_session(true)
        .build()?;

    // be sure to connect BEFORE subscribing
    async_mqtt_client.connect()?;
    async_mqtt_client.subscribe()?;

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
