use futures::future::Future;
use paho_mqtt as mqtt;
use std::error::Error;
use std::thread;
use std::time::Duration;

pub struct AsyncMqttClientBuilder {
    pub host: String,
    pub sub_topic: String,
    pub pub_topic: String,
    pub quality_of_service: i32,
    pub client_id: String,
    pub clean_session: bool,
    pub last_will_and_testament: String,
}

impl AsyncMqttClientBuilder {
    pub fn new_with_defaults() -> Self {
        let default_options = AsyncMqttClientBuilder {
            host: "test.mosquitto.org:1883".to_string(),
            sub_topic: "default".to_string(),
            pub_topic: "default".to_string(),
            last_will_and_testament: "default_lwt".to_string(),
            client_id: "default_client_id".to_string(),
            quality_of_service: 2,
            clean_session: true,
        };
        default_options
    }
    pub fn with_client_id(mut self, client_id: &str) -> Self {
        self.client_id = client_id.to_string();
        self
    }
    pub fn with_host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }
    pub fn with_subscribed_topic(mut self, sub_topic: &str) -> Self {
        self.sub_topic = sub_topic.to_string();
        self
    }
    pub fn with_publishing_topic(mut self, pub_topic: &str) -> Self {
        self.pub_topic = pub_topic.to_string();
        self
    }
    pub fn with_last_will_and_testament(mut self, last_will_and_testament: &str) -> Self {
        self.last_will_and_testament = last_will_and_testament.to_string();
        self
    }
    pub fn with_clean_session(mut self, users_choice: bool) -> Self {
        self.clean_session = users_choice;
        self
    }
    pub fn with_quality_of_service(mut self, qos: i32) -> Self {
        self.quality_of_service = qos;
        self
    }
    pub fn build(self) -> Result<AsyncMqttClient, Box<dyn Error>> {
        let create_options = mqtt::CreateOptionsBuilder::new()
            .server_uri(&self.host)
            .client_id(&self.client_id)
            .finalize();
        // let client = mqtt::AsyncClient::new(create_options)?;
        
        let sync_mqtt_client = AsyncMqttClient {
            client: mqtt::AsyncClient::new(create_options)?,
            // host: self.host,
            sub_topic: self.sub_topic,
            // pub_topic: self.pub_topic,
            quality_of_service: self.quality_of_service,
            // client_id: self.client_id,
            // clean_session: self.clean_session,
            // last_will_and_testament: self.last_will_and_testament,
        };

        Ok(sync_mqtt_client)
    }
}
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