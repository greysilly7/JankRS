#[macro_use]
extern crate rocket;

mod models;
mod routes;

use chorus::instance::ChorusUser;
use rocket::fs::{relative, FileServer};
use rocket::tokio::sync::Mutex;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::sync::Arc;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes::index::routes())
        .mount("/", routes::login::routes())
        .mount("/", routes::home::routes())
        .mount("/", routes::guild::routes())
        .mount("/", routes::channel::routes())
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
        .manage(Arc::new(Mutex::new(None::<ChorusUser>)))
        .manage(Arc::new(Mutex::new(HashMap::<String, ChorusUser>::new())))
}
