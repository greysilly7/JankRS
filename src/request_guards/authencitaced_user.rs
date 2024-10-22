use chorus::instance::ChorusUser;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::tokio::sync::Mutex;
use rocket::State;
use std::sync::Arc;

pub struct AuthenticatedUser(pub Arc<Mutex<Option<ChorusUser>>>);

#[async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_state = request
            .guard::<&State<Arc<Mutex<Option<ChorusUser>>>>>()
            .await
            .unwrap();
        let user_lock = user_state.lock().await;

        if user_lock.is_some() {
            Outcome::Success(AuthenticatedUser(user_state.inner().clone()))
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}
