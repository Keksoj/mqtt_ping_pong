use futures::future::Future;
use paho_mqtt as mqtt;
use std::error::Error;
use std::sync::mpsc;
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
        let mut client = mqtt::AsyncClient::new(create_options)?;

        let mpsc_consuming_queue = mqtt::AsyncClient::start_consuming(&mut client);

        println!("Oh hi Mark");

        let sync_mqtt_client = AsyncMqttClient {
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

pub struct AsyncMqttClient {
    pub client: mqtt::AsyncClient,
    pub host: String,
    pub sub_topic: String,
    pub pub_topic: String,
    pub quality_of_service: i32,
    pub client_id: String,
    pub clean_session: bool,
    pub last_will_and_testament: String,
    pub mpsc_consuming_queue: mpsc::Receiver<Option<mqtt::Message>>,
}

impl AsyncMqttClient {
    pub fn connect(&self) -> Result<(), Box<dyn Error>> {
        let last_will_and_testament = mqtt::MessageBuilder::new()
            .topic(String::from(&self.sub_topic))
            .payload(self.last_will_and_testament.clone())
            .finalize();

        let connect_options = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .will_message(last_will_and_testament)
            .clean_session(self.clean_session)
            .finalize();
        let connect_token = self.client.connect(connect_options).wait()?;
        println!("Connected with this token: {:?}", connect_token);
        Ok(())
    }
    pub fn subscribe(&self) -> Result<(), Box<dyn Error>> {
        let token = self
            .client
            .subscribe(self.sub_topic.clone(), self.quality_of_service)
            .wait()?;
        println!("Subscribed with this token: {:?}", token);
        Ok(())
    }

    pub fn disconnect(&self) -> Result<(), Box<dyn Error>> {
        let disconnect_token = self.client.disconnect(None).wait()?;
        println!("Disonnected with this token: {:?}", disconnect_token);
        Ok(())
    }

    pub fn reconnect(&self) -> Result<(), Box<dyn Error>> {
        let reconnect_token = self.client.reconnect().wait()?;
        println!("Reconnected with this token: {:?}", reconnect_token);
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }
    pub fn reconnect_if_needed(&self) -> Result<(), Box<dyn Error>> {
        if !self.is_connected() {
            self.reconnect()?;
            println!("Oops! We had to reconnect here.")
        }
        Ok(())
    }

    pub fn received(&self, str_to_check_for: &str) -> Result<bool, Box<dyn Error>> {
        for wrapped_message in self.mpsc_consuming_queue.iter() {
            let message = wrapped_message.unwrap();
            let payload_string: &str = std::str::from_utf8(message.payload())?;
            println!("we received: {}", payload_string);
            let answer = str_to_check_for == payload_string;
            return Ok(answer);
        }
        Ok(false)
    }

    pub fn publish(&self, content: &str) -> mqtt::errors::MqttResult<()> {
        let built_message = mqtt::MessageBuilder::new()
            .topic(&self.pub_topic)
            .payload(content)
            .qos(self.quality_of_service)
            .finalize();
        println!("Publishing '{}'", &content);
        self.client.publish(built_message).wait()
    }
}
