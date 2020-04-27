use paho_mqtt as mqtt;
use std::error::Error;
use std::sync::mpsc;
use std::time::Duration;
use std::{process, thread};
mod variables;

const HOST: &str = "test.mosquitto.org:1883";
const SUBSCRIBED_TOPICS: [&str; 2] = ["test", "ping-ask"];
const QUALITIES_OF_SERVICE: [i32; 2] = [2, 2];

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating a synchronous mqtt client...");
    let mut client = new_client(HOST)?;

    // initialize the consumer before connecting. This puts received messages
    // in a mpsc queue (multiple producer single consumer)
    let mpsc_consumer = client.start_consuming();

    establish_connection(&client)?;
    listen_to_the_ping(&client, &mpsc_consumer)?;
    if client.is_connected() {
        unsubscribe_and_disconnect(&client)?;
    }
    
    Ok(())
}

fn new_client(host: &str) -> mqtt::errors::MqttResult<mqtt::Client> {
    let create_options = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id("sync_pong")
        .finalize();
    let client = mqtt::Client::new(create_options)?;
    Ok(client)
}

fn establish_connection(client: &mqtt::Client) -> mqtt::errors::MqttResult<()> {
    let last_will_and_testament = mqtt::MessageBuilder::new()
        .topic("ping-ask")
        .payload("the synchronized ponger lost connection")
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
                    "Subscribing to the topics {:?} with QoS {:?}...",
                    &SUBSCRIBED_TOPICS, &QUALITIES_OF_SERVICE
                );

                match client.subscribe_many(&SUBSCRIBED_TOPICS, &QUALITIES_OF_SERVICE) {
                    Ok(qos) => Ok(println!("QoS granted: {:?}", qos)),
                    Err(error) => {
                        println!("Error subscribing to topics: {}", error);
                        client.disconnect(None)?;
                        process::exit(1);
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

fn listen_to_the_ping(
    client: &mqtt::Client,
    mpsc_consumer: &mpsc::Receiver<Option<mqtt::Message>>,
) -> Result<(), Box<dyn Error>> {
    println!("Waiting for the ping, consuming the mpsc queue...");
    for wrapped_message in mpsc_consumer.iter() {
        let message = wrapped_message.unwrap();
        let payload_string: &str = std::str::from_utf8(message.payload())?;
        match payload_string {
            "ping" => {
                println!("{}\nWe received the ping!", payload_string);
                publish_pong(client)?;
            }
            _ => println!("{}\nThat wasn't a pong...", payload_string),
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

fn publish_pong(client: &mqtt::Client) -> mqtt::errors::MqttResult<()> {
    let pong_message = mqtt::MessageBuilder::new()
        .topic("pong-response")
        .payload("pong")
        .qos(2)
        .finalize();

    println!("Ponging back.");
    client.publish(pong_message)
}

fn unsubscribe_and_disconnect(client: &mqtt::Client) -> mqtt::errors::MqttResult<()> {
    println!("Disconnecting");
    client.unsubscribe_many(&SUBSCRIBED_TOPICS)?;
    client.disconnect(None)
}