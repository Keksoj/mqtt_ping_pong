use paho_mqtt as mqtt;
use std::error::Error;
use std::sync::mpsc;
use std::time::Duration;
use std::{process, thread};

pub struct SyncMqttClientBuilder {
    pub host: String,
    pub sub_topic: String,
    pub pub_topic: String,
    pub quality_of_service: i32,
    pub client_id: String,
    pub clean_session: bool,
    pub last_will_and_testament: String,
}

impl SyncMqttClientBuilder {
    pub fn new_with_defaults() -> Self {
        let default_options = SyncMqttClientBuilder {
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
    pub fn build(self) -> Result<SyncMqttClient, Box<dyn Error>> {
        let create_options = mqtt::CreateOptionsBuilder::new()
            .server_uri(&self.host)
            .client_id(&self.client_id)
            .finalize();
        let mut client = mqtt::Client::new(create_options)?;
        let mpsc_consuming_queue = mqtt::Client::start_consuming(&mut client);
        let sync_mqtt_client = SyncMqttClient {
            client,
            mpsc_consuming_queue,
            host: self.host,
            sub_topic: self.sub_topic,
            pub_topic: self.pub_topic,
            quality_of_service: self.quality_of_service,
            client_id: self.client_id,
            clean_session: self.clean_session,
            last_will_and_testament: self.last_will_and_testament,
        };

        Ok(sync_mqtt_client)
    }
}

pub struct SyncMqttClient {
    pub client: mqtt::Client,
    pub host: String,
    pub sub_topic: String,
    pub pub_topic: String,
    pub quality_of_service: i32,
    pub client_id: String,
    pub clean_session: bool,
    pub last_will_and_testament: String,
    pub mpsc_consuming_queue: mpsc::Receiver<Option<mqtt::Message>>,
}

impl SyncMqttClient {
    pub fn establish_connection(&mut self) -> mqtt::errors::MqttResult<()> {
        let built_lwt = mqtt::MessageBuilder::new()
            .topic(&self.sub_topic)
            .payload(self.last_will_and_testament.clone())
            .finalize();

        let connection_options = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(self.clean_session)
            .will_message(built_lwt)
            .finalize();

        println!("Connecting to the broker '{}'", self.host);

        match self.client.connect(connection_options) {
            Ok((server_uri, mqtt_version, session_present)) => {
                println!(
                    "Connected to '{}' with MQTT version {}",
                    server_uri, mqtt_version
                );
                if session_present {
                    return Ok(println!("We already have a session present!"));
                }
                println!(
                    "Subscribing to the topic {} with QoS {}...",
                    self.sub_topic, self.quality_of_service
                );
                match self
                    .client
                    .subscribe(&self.sub_topic, self.quality_of_service)
                {
                    Ok(qos) => Ok(println!("QoS granted: {:?}", qos)),
                    Err(error) => {
                        println!("Error subscribing to topics: {}", error);
                        self.client.disconnect(None)?;
                        process::exit(1)
                    }
                }
            }
            Err(error) => {
                println!("error connecting to {}: {:?}", self.host, error);
                // todo: put this exit call in the main()
                process::exit(1);
            }
        }
    }

    pub fn publish(&mut self, content: &str) -> mqtt::errors::MqttResult<()> {
        let built_message = mqtt::MessageBuilder::new()
            .topic(&self.pub_topic)
            .payload(content)
            .qos(self.quality_of_service)
            .finalize();

        self.client.publish(built_message)?;
        println!("We published the {}!", content);
        Ok(())
    }

    pub fn reestablish_connection(&mut self) {
        if !self.client.is_connected() {
            println!("Connection lost. Waiting to reestablish the connection");
            for _ in 0..12 {
                thread::sleep(Duration::from_millis(5000));
                if self.client.reconnect().is_ok() {
                    println!("Successfully reconnected");
                    break;
                }
            }
            println!("Unable to reconnect after several attempts.");
        }
    }

    pub fn received(&self, str_to_check_for: &str) -> Result<bool, Box<dyn Error>> {
        println!("we are here");
        for wrapped_message in self.mpsc_consuming_queue.iter() {
            let message = wrapped_message.unwrap();
            let payload_string: &str = std::str::from_utf8(message.payload())?;
            println!("we received: {}", payload_string);
            let answer = str_to_check_for == payload_string;
            return Ok(answer);
        }
        Ok(false)
    }
}
