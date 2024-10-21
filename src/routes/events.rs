use chorus::instance::ChorusUser;
use chorus::types::MessageCreate;
use pubserve::Subscriber;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::error::RecvError;
use rocket::tokio::sync::broadcast::Sender;
use rocket::tokio::sync::Mutex;
use rocket::State;
use rocket::{Route, Shutdown};
use std::sync::Arc;

#[derive(Debug)]
pub struct MessageCreateObserver {
    queue: Sender<MessageCreate>,
}

#[async_trait]
impl Subscriber<MessageCreate> for MessageCreateObserver {
    async fn update(&self, data: &MessageCreate) {
        let _ = self.queue.send(data.clone());
        println!("Observed Ready!");
    }
}

#[get("/events")]
pub async fn event_stream(
    user: &State<Arc<Mutex<Option<ChorusUser>>>>,
    queue: &State<Sender<MessageCreate>>,
    mut end: Shutdown,
) -> EventStream![] {
    let mut rx = queue.subscribe();

    let mut user_lock: rocket::tokio::sync::MutexGuard<'_, Option<ChorusUser>> = user.lock().await;
    if let Some(chorus_user) = user_lock.as_mut() {
        // Create an instance of our observer
        let observer = MessageCreateObserver {
            queue: queue.inner().clone(),
        };

        // Share ownership of the observer with the gateway
        let shared_observer = Arc::new(observer);

        chorus_user
            .gateway
            .events
            .lock()
            .await
            .message
            .create
            .subscribe(shared_observer);
    }

    EventStream! {
      loop {
        let msg: MessageCreate = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);

      }
    }
}

pub fn routes() -> Vec<Route> {
    routes![event_stream]
}
