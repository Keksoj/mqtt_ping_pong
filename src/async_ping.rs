use futures::Future;
use paho_mqtt as mqtt;
mod variables;
use std::thread;
use std::time::Duration;

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
        // .client_id("sync_ping")
        .persistence(mqtt::PersistenceType::None)
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
        .payload("the asynchronized ponger lost connection")
        .finalize();

    let connect_options = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        .will_message(last_will_and_testament)
        .clean_session(true)
        .finalize();

    println!("connecting to the broker {}", &host);
    client.connect_with_callbacks(connect_options, connect_success_cb, connect_failure_cb).wait();


    //         _             _
    //   _ __ (_)_ __   __ _(_)_ __   __ _
    //  | '_ \| | '_ \ / _` | | '_ \ / _` |
    //  | |_) | | | | | (_| | | | | | (_| |
    //  | .__/|_|_| |_|\__, |_|_| |_|\__, |
    //  |_|            |___/         |___/
    let ping_message = mqtt::MessageBuilder::new()
        .topic("ping-ask")
        .payload("ping")
        .qos(2)
        .finalize();

    let delivery_token = client.publish(ping_message);
    match delivery_token.wait() {
        Ok(()) => println!("successfully pinged!"),
        Err(error) => println!("Error while pinging: {}", error),
    }

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

const TOPIC: &str = "pong-response";
const QUALITY_OF_SERVICE: i32 = 2;

fn initiate_reconnection(client: &mqtt::AsyncClient) {
    println!("Connection lost. Attempting to reconnect");
    thread::sleep(Duration::from_millis(2500));
    client.reconnect_with_callbacks(connect_success_cb, connect_failure_cb);
}

fn connect_success_cb(client: &mqtt::AsyncClient, _message_id: u16) {
    println!("Connection succeeded");
    // subscribe to the desired topics
    client.subscribe(TOPIC, QUALITY_OF_SERVICE);
    println!("Subscribing to topics: {:?}", TOPIC);
}

// this is a sleep & retry that calls itself recursively
fn connect_failure_cb(client: &mqtt::AsyncClient, _message_id: u16, return_code: i32) {
    println!(
        "Connection attempt failed with return code {}.\n",
        return_code
    );
    thread::sleep(Duration::from_millis(2500));
    client.reconnect_with_callbacks(connect_success_cb, connect_failure_cb);
}

fn handle_messages(_client: &mqtt::AsyncClient, wrapped_message: Option<mqtt::Message>) {
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
}
