use paho_mqtt as mqtt;
use std::thread;
use std::time::Duration;
mod variables;

fn main() {
    //                       _         _   _                 _ _            _
    //    ___ _ __ ___  __ _| |_ ___  | |_| |__   ___    ___| (_) ___ _ __ | |_
    //   / __| '__/ _ \/ _` | __/ _ \ | __| '_ \ / _ \  / __| | |/ _ \ '_ \| __|
    //  | (__| | |  __/ (_| | ||  __/ | |_| | | |  __/ | (__| | |  __/ | | | |_
    //   \___|_|  \___|\__,_|\__\___|  \__|_| |_|\___|  \___|_|_|\___|_| |_|\__|
    //
    let host: String = variables::HOST.to_string();

    let create_options = mqtt::CreateOptionsBuilder::new()
        .server_uri(&host)
        .client_id("sync_pong")
        .finalize();

    let mut client = match mqtt::AsyncClient::new(create_options) {
        Ok(client) => client,
        Err(error) => panic!("error creating the client: {:?}", error),
    };

    client.set_connection_lost_callback(initiate_reconnection);

    client.set_message_callback(handle_messages);

    //                                   _
    //    ___ ___  _ __  _ __   ___  ___| |_
    //   / __/ _ \| '_ \| '_ \ / _ \/ __| __|
    //  | (_| (_) | | | | | | |  __/ (__| |_
    //   \___\___/|_| |_|_| |_|\___|\___|\__|

    let last_will_and_testament = mqtt::MessageBuilder::new()
        .topic("pong-response")
        .payload("the synchronized ponger lost connection")
        .finalize();

    let connect_options = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        .will_message(last_will_and_testament)
        .clean_session(true)
        .finalize();

    println!("connecting to the broker {}", &host);
    client.connect_with_callbacks(connect_options, connect_success_cb, connect_failure_cb);

    // wait for incoming messages
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}

//             _ _ _                _
//    ___ __ _| | | |__   __ _  ___| | _____
//   / __/ _` | | | '_ \ / _` |/ __| |/ / __|
//  | (_| (_| | | | |_) | (_| | (__|   <\__ \
//   \___\__,_|_|_|_.__/ \__,_|\___|_|\_\___/

const SUBSCRIBED_TOPICS: &[&str] = &["test", "ping-ask"];
const QUALITIES_OF_SERVICE: &[i32; 2] = &[2, 2];

fn initiate_reconnection(client: &mqtt::AsyncClient) {
    println!("Connection lost. Attempting to reconnect");
    thread::sleep(Duration::from_millis(2500));
    client.reconnect_with_callbacks(connect_success_cb, connect_failure_cb);
}

fn connect_success_cb(client: &mqtt::AsyncClient, _message_id: u16) {
    println!("Connection succeeded");
    // subscribe to the desired topics
    client.subscribe_many(SUBSCRIBED_TOPICS, QUALITIES_OF_SERVICE);
    println!("Subscribing to topics: {:?}", SUBSCRIBED_TOPICS);
}

// this is a sleep & retry that calls itself recursively
fn connect_failure_cb(client: &mqtt::AsyncClient, _message_id: u16, error_code: i32) {
    println!(
        "Connection attempt failed with error code {}.\n",
        error_code
    );
    thread::sleep(Duration::from_millis(2500));
    client.reconnect_with_callbacks(connect_success_cb, connect_failure_cb);
}

fn handle_messages(client: &mqtt::AsyncClient, wrapped_message: Option<mqtt::Message>) {
    match wrapped_message {
        Some(message) => {
            let topic = message.topic();
            let payload_str = message.payload_str();
            println!(
                "On the topic '{}', we received this message: '{}' ",
                topic, payload_str
            );
        }
        None => println!("Well.... nothing"),
    }
}
