use chorus::instance::ChorusUser;
use chorus::types::MessageCreate;
use pubserve::Subscriber;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::mpsc::{self, Receiver, Sender};
use rocket::tokio::sync::Mutex;
use rocket::Route;
use rocket::Shutdown;
use rocket::State;
use std::sync::Arc;

#[derive(Debug)]
pub struct MessageEventObserver {
    sender: Sender<MessageCreate>,
    receiver: Mutex<Receiver<MessageCreate>>,
}

#[async_trait]
impl Subscriber<MessageCreate> for MessageEventObserver {
    // After we subscribe to an event this function is called every time we receive it
    async fn update(&self, data: &MessageCreate) {
        println!("Received message event: {:?}", data);
        if let Err(e) = self.sender.send(data.clone()).await {
            eprintln!("Failed to send message event: {:?}", e);
        }
    }
}

pub struct AppState {
    observer: Option<Arc<MessageEventObserver>>,
}

/// Initializes the observer and returns the application state.
pub async fn initialize_observer(user: Arc<Mutex<Option<ChorusUser>>>) -> AppState {
    println!("Initializing observer");
    let (tx, rx) = mpsc::channel(100);
    let observer = MessageEventObserver {
        sender: tx,
        receiver: Mutex::new(rx),
    };
    let shared_observer = Arc::new(observer);

    let mut user_lock = user.lock().await;
    if let Some(chorus_user) = user_lock.as_mut() {
        chorus_user
            .gateway
            .events
            .lock()
            .await
            .message
            .create
            .subscribe(shared_observer.clone());
    }

    AppState {
        observer: Some(shared_observer),
    }
}

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue sent by the `post` handler.

#[get("/events")]
pub async fn events(state: &State<AppState>, mut end: Shutdown) -> EventStream![] {
    let (tx, mut rx) = mpsc::channel(100);

    let observer = state.observer.clone();

    EventStream! {
        if let Some(observer) = observer {
            // Forward messages from the existing receiver to the new channel
            let observer_clone = observer.clone();
            let new_tx = tx.clone();

            rocket::tokio::spawn(async move {
                let mut existing_rx = observer_clone.receiver.lock().await;
                while let Some(event) = existing_rx.recv().await {
                    if new_tx.send(event).await.is_err() {
                        break;
                    }
                }
            });

            loop {
                select! {
                    event = rx.recv() => {
                        if let Some(event) = event {
                            println!("Sending event: {:?}", event);
                            yield Event::json(&event);
                        } else {
                            break;
                        }
                    },
                    _ = &mut end => {
                        break;
                    }
                }
            }
        } else {
            yield Event::data("User not logged in");
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![events]
}
