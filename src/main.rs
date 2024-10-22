#[macro_use]
extern crate rocket;

mod models;
mod request_guards;
mod routes;

use chorus::instance::ChorusUser;
use chorus::types::MessageCreate;
use rocket::fs::{relative, FileServer};
use rocket::tokio::sync::{broadcast, Mutex};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::sync::Arc;

#[launch]
async fn rocket() -> _ {
    let user_state = Arc::new(Mutex::new(None::<ChorusUser>));
    let message_state = Arc::new(Mutex::new(
        HashMap::<String, Vec<HashMap<String, String>>>::new(),
    ));
    let (tx, _rx) = broadcast::channel::<MessageCreate>(100);

    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes::index::routes())
        .mount("/", routes::login::routes())
        .mount("/", routes::home::routes())
        .mount("/", routes::guild::routes())
        .mount("/", routes::channel::routes())
        .mount("/", routes::events::routes())
        .mount("/static", FileServer::from(relative!("static")))
        .manage(user_state)
        .manage(message_state)
        .manage(tx)
}
