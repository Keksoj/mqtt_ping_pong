use paho_mqtt as mqtt;
use std::time::Duration;
use std::{process, thread};

fn main() {
    //                       _         _   _                 _ _            _
    //    ___ _ __ ___  __ _| |_ ___  | |_| |__   ___    ___| (_) ___ _ __ | |_
    //   / __| '__/ _ \/ _` | __/ _ \ | __| '_ \ / _ \  / __| | |/ _ \ '_ \| __|
    //  | (__| | |  __/ (_| | ||  __/ | |_| | | |  __/ | (__| | |  __/ | | | |_
    //   \___|_|  \___|\__,_|\__\___|  \__|_| |_|\___|  \___|_|_|\___|_| |_|\__|
    //
    let host: String = "test.mosquitto.org:1883".to_string();

    let create_options = mqtt::CreateOptionsBuilder::new()
        .server_uri(&host)
        .client_id("sync_pong")
        .finalize();

    let mut client = match mqtt::Client::new(create_options) {
        Ok(client) => client,
        Err(error) => panic!("error creating the client: {:?}", error),
    };
    println!("I am the synchronous pong program.");
    //                                   _
    //    ___ ___  _ __  _ __   ___  ___| |_
    //   / __/ _ \| '_ \| '_ \ / _ \/ __| __|
    //  | (_| (_) | | | | | | |  __/ (__| |_
    //   \___\___/|_| |_|_| |_|\___|\___|\__|
    // initialize the consumer before connecting. This puts received messages
    // in a mpsc queue (multiple producer single consumer)
    let mpsc_consumer = client.start_consuming();

    let last_will_and_testament = mqtt::MessageBuilder::new()
        .topic("ping-ask")
        .payload("the synchronized ponger lost connection")
        .finalize();

    let connection_options = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .will_message(last_will_and_testament)
        .finalize();

    let subscribed_topics = ["test", "ping-ask"];
    let qualities_of_service: [i32; 2] = [2, 2];

    println!("Connecting to the broker '{}'", &host);

    // initiate and check the connection
    match client.connect(connection_options) {
        Ok(response) => {
            let (server_uri, ver, session_present) = response;
            println!("Connected to '{}' with MQTT version {}", server_uri, ver);
            if !session_present {
                println!(
                    "Subscribing to the topics {:?} with QoS {:?}...",
                    &subscribed_topics,
                    &qualities_of_service
                );

                match client.subscribe_many(&subscribed_topics, &qualities_of_service) {
                    Ok(qos) => println!("QoS granted: {:?}", qos),
                    Err(error) => {
                        println!("Error subscribing to topics: {}", error);
                        client.disconnect(None).unwrap();
                        process::exit(1);
                    }
                }
            } else {
                println!("We already have a session present!");
            }
        }
        Err(error) => {
            println!("error connecting to {}: {:?}", &host, error);
            process::exit(1);
        }
    }
    //                           _
    //   _ __   ___  _ __   __ _(_)_ __   __ _
    //  | '_ \ / _ \| '_ \ / _` | | '_ \ / _` |
    //  | |_) | (_) | | | | (_| | | | | | (_| |
    //  | .__/ \___/|_| |_|\__, |_|_| |_|\__, |
    //  |_|                |___/         |___/
    println!("Waiting for messages...");
    // this consumes the mpsc queue
    for wrapped_message in mpsc_consumer.iter() {
        let message = wrapped_message.unwrap();
        let payload_string: &str = match std::str::from_utf8(message.payload()) {
            Ok(str) => str,
            Err(error) => panic!("Couldn't unpack the message payload: {}", error),
        };
        println!("{}", payload_string);
        match payload_string {
            "ping" => {
                println!("We got a ping message! Let's pong back.");
                let pong_message = mqtt::MessageBuilder::new()
                    .topic("pong-response")
                    .payload("pong")
                    .qos(2)
                    .finalize();

                match client.publish(pong_message) {
                    Ok(()) => println!("successfully ponged!"),
                    Err(error) => println!("Error while ponging: {:?}", error),
                }
            }
            _ => println!("That wasn't a ping message..."),
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
    //       _ _                                     _
    //    __| (_)___  ___ ___  _ __  _ __   ___  ___| |_
    //   / _` | / __|/ __/ _ \| '_ \| '_ \ / _ \/ __| __|
    //  | (_| | \__ \ (_| (_) | | | | | | |  __/ (__| |_
    //   \__,_|_|___/\___\___/|_| |_|_| |_|\___|\___|\__|
    if client.is_connected() {
        println!("Disconnecting");
        client.unsubscribe_many(&subscribed_topics).unwrap();
        client.disconnect(None).unwrap();
    }
    println!("Exiting");
}
