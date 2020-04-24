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
        .client_id("sync_ping")
        .finalize();

    let mut client = match mqtt::Client::new(create_options) {
        Ok(client) => client,
        Err(error) => panic!("error creating the client: {:?}", error),
    };

    println!("I am the synchronous ping program.");
    // 5 secs timeouts for synchronous calls
    client.set_timeout(Duration::from_secs(5));

    //                                   _
    //    ___ ___  _ __  _ __   ___  ___| |_
    //   / __/ _ \| '_ \| '_ \ / _ \/ __| __|
    //  | (_| (_) | | | | | | |  __/ (__| |_
    //   \___\___/|_| |_|_| |_|\___|\___|\__|
    // initialize the consumer before connecting. This puts received messages
    // in a mpsc queue (multiple producer single consumer)
    let mpsc_consumer = client.start_consuming();

    let last_will_and_testament = mqtt::MessageBuilder::new()
        .topic("pong-response")
        .payload("the synchronized pinger lost connection")
        .finalize();

    let connection_options = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .will_message(last_will_and_testament)
        .finalize();

    println!("Connecting to the broker '{}'", &host);

    // initiate and check the connection
    match client.connect(connection_options) {
        Ok(response) => {
            let (server_uri, ver, session_present) = response;
            println!("Connected to '{}' with MQTT version {}", server_uri, ver);
            if !session_present {
                println!("Subscribing to topic 'pong-response' with QoS 2",);

                match client.subscribe(&"pong-response", 2) {
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
    //         _             _
    //   _ __ (_)_ __   __ _(_)_ __   __ _
    //  | '_ \| | '_ \ / _` | | '_ \ / _` |
    //  | |_) | | | | | (_| | | | | | (_| |
    //  | .__/|_|_| |_|\__, |_|_| |_|\__, |
    //  |_|            |___/         |___/
    let msg = mqtt::MessageBuilder::new()
        .topic("ping-ask")
        .payload("ping")
        .qos(2)
        .finalize();

    match client.publish(msg) {
        Ok(()) => println!("Pinging successful!"),
        Err(error) => panic!("The pinging failed : {}", error),
    }
    //   _ _     _             _
    //  | (_)___| |_ ___ _ __ (_)_ __   __ _
    //  | | / __| __/ _ \ '_ \| | '_ \ / _` |
    //  | | \__ \ ||  __/ | | | | | | | (_| |
    //  |_|_|___/\__\___|_| |_|_|_| |_|\__, |
    //                                 |___/
    println!("Waiting for the pong...");
    // this consumes the mpsc queue
    for wrapped_message in mpsc_consumer.iter() {
        let message = wrapped_message.unwrap();
        let payload_string: &str = match std::str::from_utf8(message.payload()) {
            Ok(str) => str,
            Err(error) => panic!("Couldn't unpack the message payload: {}", error),
        };
        println!("{}", payload_string);
        match payload_string {
            "pong" => println!("We received the pong!"),
            _ => println!("That wasn't a pong..."),
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
    client.disconnect(None).unwrap();
}
