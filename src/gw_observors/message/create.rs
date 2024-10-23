use chorus::types::MessageCreate;
use pubserve::Subscriber;
use rocket::tokio::sync::broadcast::Sender;

#[derive(Debug)]
pub struct MessageCreateObserver {
    pub queue: Sender<MessageCreate>,
}

#[async_trait]
impl Subscriber<MessageCreate> for MessageCreateObserver {
    async fn update(&self, data: &MessageCreate) {
        let _ = self.queue.send(data.clone());
        println!("Observed MessageCreate!");
    }
}
