use futures::future::Future;
use paho_mqtt as mqtt;
use std::error::Error;
use std::thread;
use std::time::Duration;

pub struct AsyncMqttClient {
    pub client: mqtt::AsyncClient,
    pub sub_topic: String,
    pub quality_of_service: i32,
    // todo: 
    // pub connection_lost_cb: FnMut(&AsyncClient) + 'static,
    // pub handle_messages_callback: FnMut(&AsyncClient, Option<Message>) + 'static,
    // etc.
}

impl AsyncMqttClient {
    pub fn new(host: &str) -> mqtt::errors::MqttResult<Self> {
        let create_options = mqtt::CreateOptionsBuilder::new()
            .server_uri(host)
            .client_id("async_pong")
            .persistence(mqtt::PersistenceType::None)
            .finalize();
        let async_mqtt_client = AsyncMqttClient {
            client: mqtt::AsyncClient::new(create_options)?,
            sub_topic: String::from("ping_ask"),
            quality_of_service: 2,
        };
        Ok(async_mqtt_client)
    }

    // fn initiate_reconnection(&mut self) {
    //     println!("Connection lost. Attempting to reconnect");
    //     thread::sleep(Duration::from_millis(2500));
    //     self.client
    //         .reconnect_with_callbacks(Self::connect_success_cb, Self::connect_failure_cb);
    // }

    // fn connect_success_cb(client: &mut Self.client, _message_id: u16) {
    //     println!("Connection succeeded");
    //     // subscribe to the desired topics
    //     client
    //         .subscribe(self.sub_topic, self.quality_of_service);
    //     println!("Subscribing to topics: {}", self.sub_topic);
    // }

    // // this is a sleep & retry that calls itself recursively
    // fn connect_failure_cb(client: &mut Self.client, _message_id: u16, return_code: i32) {
    //     println!(
    //         "Connection attempt failed with return code {}.\n",
    //         return_code
    //     );
    //     thread::sleep(Duration::from_millis(2500));
    //     client
    //         .reconnect_with_callbacks(connect_success_cb, connect_failure_cb);
    // }

    fn publish_pong(&self, topic: &str, message: &str) -> mqtt::errors::MqttResult<()> {
        let ping_message = mqtt::MessageBuilder::new()
            .topic(topic)
            .payload(message)
            .qos(2)
            .finalize();
        println!("ponging");
        self.client.publish(ping_message).wait()
    }
}

// fn initiate_reconnection(client: &mqtt::AsyncClient) {
//     println!("Connection lost. Attempting to reconnect");
//     thread::sleep(Duration::from_millis(2500));
//     client.reconnect_with_callbacks(connect_success_cb, connect_failure_cb);
// }

// fn connect_success_cb(client: &mqtt::AsyncClient, _message_id: u16) {
//     println!("Connection succeeded");
//     // subscribe to the desired topics
//     client.subscribe_many(SUBSCRIBED_TOPICS, QUALITIES_OF_SERVICE);
//     println!("Subscribing to topics: {:?}", SUBSCRIBED_TOPICS);
// }

// // this is a sleep & retry that calls itself recursively
// fn connect_failure_cb(client: &mqtt::AsyncClient, _message_id: u16, return_code: i32) {
//     println!(
//         "Connection attempt failed with return code {}.\n",
//         return_code
//     );
//     thread::sleep(Duration::from_millis(2500));
//     client.reconnect_with_callbacks(connect_success_cb, connect_failure_cb);
// }