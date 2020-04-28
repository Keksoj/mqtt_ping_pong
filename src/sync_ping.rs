use std::error::Error;
mod sync_lib;
use sync_lib::{SyncMqttClient, SyncMqttClientBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating a synchronous mqtt client...");
    
    let mut sync_mqtt_client = SyncMqttClientBuilder::new_with_defaults()
        .with_host("test.mosquitto.org:1883")
        .with_client_id("Synchronised pinger")
        .with_publishing_topic("ping-ask")
        .with_subscribed_topic("pong-response")
        .with_last_will_and_testament("the synchronised pinger lost the connection")
        .with_quality_of_service(2)
        .with_clean_session(true)
        .build()?;

    sync_mqtt_client.establish_connection()?;

    sync_mqtt_client.publish("ping")?;

    loop {
        if sync_mqtt_client.received("pong")? {
            println!("Hurray my friend ponged back!");
            break;
        }
        sync_mqtt_client.reestablish_connection();
    }

    sync_mqtt_client.client.disconnect(None)?;
    Ok(())
}
