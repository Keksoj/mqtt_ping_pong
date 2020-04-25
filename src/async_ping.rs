use futures::Future;
use paho_mqtt as mqtt;
mod variables;
// use std::time::Duration;
// use std::{process, thread};

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

    let client = match mqtt::AsyncClient::new(create_options) {
        Ok(client) => client,
        Err(error) => panic!("error creating the client: {:?}", error),
    };

    //                                   _
    //    ___ ___  _ __  _ __   ___  ___| |_
    //   / __/ _ \| '_ \| '_ \ / _ \/ __| __|
    //  | (_| (_) | | | | | | |  __/ (__| |_
    //   \___\___/|_| |_|_| |_|\___|\___|\__|
    // default connecting options
    let connection_options = mqtt::ConnectOptions::new();
    //
    let connection_token = client.connect(connection_options);
    if let Err(error) = connection_token.wait() {
        panic!("Couldn't connect the client: {}", error);
    }

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
    //       _ _                                     _
    //    __| (_)___  ___ ___  _ __  _ __   ___  ___| |_
    //   / _` | / __|/ __/ _ \| '_ \| '_ \ / _ \/ __| __|
    //  | (_| | \__ \ (_| (_) | | | | | | |  __/ (__| |_
    //   \__,_|_|___/\___\___/|_| |_|_| |_|\___|\___|\__|
    let disconnection_token = client.disconnect(None);
    disconnection_token.wait().unwrap();
}
