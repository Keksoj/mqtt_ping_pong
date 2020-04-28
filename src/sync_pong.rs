use std::error::Error;
mod sync_lib;
use sync_lib::{SyncMqttClient, SyncMqttClientBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating a synchronous mqtt client...");
    let mut sync_mqtt_client = SyncMqttClientBuilder::new_with_defaults()
        .with_host("test.mosquitto.org:1883")
        .with_client_id("sync_pong")
        .with_publishing_topic("pong-response")
        .with_subscribed_topic("ping-ask")
        .with_last_will_and_testament("the synchronised ponger lost the connection")
        .with_quality_of_service(2)
        .with_clean_session(true)
        .build()?;

    sync_mqtt_client.establish_connection()?;

    loop {
        if sync_mqtt_client.received("ping")? {
            sync_mqtt_client.publish("pong")?;
            println!("Our job here is done");
            break;
        }
        sync_mqtt_client.reestablish_connection();
    }

    sync_mqtt_client.client.disconnect(None)?;

    Ok(())
}
