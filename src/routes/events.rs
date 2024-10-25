use chorus::types::MessageCreate;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::error::RecvError;
use rocket::tokio::sync::broadcast::Sender;
use rocket::State;
use rocket::{Route, Shutdown};
use std::sync::Arc;

use crate::gw_observors::message::create::MessageCreateObserver;
use crate::request_guards::authencitaced_user::AuthenticatedUser;

#[get("/events")]
pub async fn event_stream(
    user: AuthenticatedUser,
    queue: &State<Sender<Arc<MessageCreate>>>,
    mut end: Shutdown,
) -> EventStream![] {
    let mut rx = queue.subscribe();

    let mut user_lock = user.0.lock().await;
    if let Some(chorus_user) = user_lock.as_mut() {
        let mut events_lock = chorus_user.gateway.events.lock().await;
        if !events_lock.message.create.has_subscribers() {
            let observer = Arc::new(MessageCreateObserver {
                queue: (*queue).clone(),
            });
            events_lock.message.create.subscribe(observer);
        }
    }

    EventStream! {
        loop {
            let msg: Arc<MessageCreate> = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&*msg);
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![event_stream]
}
