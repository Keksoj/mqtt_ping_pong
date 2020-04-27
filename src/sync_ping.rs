use paho_mqtt as mqtt;
use std::error::Error;
use std::sync::mpsc;
use std::time::Duration;
use std::{process, thread};
mod variables;

const HOST: &str = "test.mosquitto.org:1883";
const SUBSCRIBED_TOPIC: &str = "pong-response";
const QUALITY_OF_SERVICE: i32 = 2;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating a synchronous mqtt client...");
    let mut client = new_client(HOST)?;

    // initialize the consumer before connecting. This puts received messages
    // in a mpsc queue (multiple producer single consumer)
    let mpsc_consumer = client.start_consuming();

    establish_connection(&client)?;

    publish_ping(&client)?;

    listen_to_the_pong(&client, &mpsc_consumer)?;

    client.disconnect(None)?;
    Ok(())
}

fn new_client(host: &str) -> mqtt::errors::MqttResult<mqtt::Client> {
    let create_options = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id("sync_ping")
        .finalize();
    let client = mqtt::Client::new(create_options)?;
    Ok(client)
}

fn establish_connection(client: &mqtt::Client) -> mqtt::errors::MqttResult<()> {
    let last_will_and_testament = mqtt::MessageBuilder::new()
        .topic(SUBSCRIBED_TOPIC)
        .payload("the synchronized pinger lost connection")
        .finalize();

    let connection_options = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .will_message(last_will_and_testament)
        .finalize();

    println!("Connecting to the broker '{}'", HOST);

    match client.connect(connection_options) {
        Ok((server_uri, ver, session_present)) => {
            println!("Connected to '{}' with MQTT version {}", server_uri, ver);
            if !session_present {
                println!(
                    "Subscribing to topic 'pong-response' with QoS {}",
                    &QUALITY_OF_SERVICE
                );

                match client.subscribe(&SUBSCRIBED_TOPIC, QUALITY_OF_SERVICE) {
                    Ok(qos) => Ok(println!("QoS granted: {:?}", qos)),
                    Err(error) => {
                        println!("Error subscribing to topics: {}", error);
                        client.disconnect(None)?;
                        Err(error)
                    }
                }
            } else {
                return Ok(println!("We already have a session present!"));
            }
        }
        Err(error) => {
            println!("error connecting to {}: {:?}", HOST, error);
            process::exit(1);
        }
    }
}

fn publish_ping(client: &mqtt::Client) -> mqtt::errors::MqttResult<()> {
    let ping_message = mqtt::MessageBuilder::new()
        .topic("ping-ask")
        .payload("ping")
        .qos(2)
        .finalize();

    println!("Sending the ping.");
    client.publish(ping_message)
}

fn listen_to_the_pong(
    client: &mqtt::Client,
    mpsc_consumer: &mpsc::Receiver<Option<mqtt::Message>>,
) -> Result<(), Box<dyn Error>> {
    println!("Waiting for the pong, consuming the mpsc queue...");
    for wrapped_message in mpsc_consumer.iter() {
        let message = wrapped_message.unwrap();
        let payload_string: &str = std::str::from_utf8(message.payload())?;
        match payload_string {
            "pong" => println!("{}\nWe received the pong!", payload_string),
            _ => println!("{}That wasn't a pong...", payload_string),
        }
        if !client.is_connected() {
            println!("Connection lost. Waiting to reestablish the connection");
            for _ in 0..12 {
                thread::sleep(Duration::from_millis(5000));
                if client.reconnect().is_ok() {
                    println!("Successfully reconnected");
                    break;
                }
            }
            println!("Unable to reconnect after several attempts.");
            break;
        }
    }
    Ok(())
}
