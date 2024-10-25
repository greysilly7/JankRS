use chorus::types::MessageCreate;
use pubserve::Subscriber;
use rocket::tokio::sync::broadcast::Sender;
use std::sync::Arc;

#[derive(Debug)]
pub struct MessageCreateObserver {
    pub queue: Sender<Arc<MessageCreate>>,
}

#[async_trait]
impl Subscriber<MessageCreate> for MessageCreateObserver {
    async fn update(&self, data: &MessageCreate) {
        let data_arc = Arc::new(data.clone());
        let _ = self.queue.send(data_arc);
        println!("Observed MessageCreate!");
    }
}
