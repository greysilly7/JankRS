#[macro_use]
extern crate rocket;

mod models;
mod routes;

use chorus::instance::ChorusUser;
use rocket::fs::{relative, FileServer};
use rocket::tokio::sync::Mutex;
use rocket_dyn_templates::Template;
use routes::events::initialize_observer;
use std::collections::HashMap;
use std::sync::Arc;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes::index::routes())
        .mount("/", routes::login::routes())
        .mount("/", routes::home::routes())
        .mount("/", routes::guild::routes())
        .mount("/", routes::channel::routes())
        .mount("/", routes::events::routes())
        .mount("/static", FileServer::from(relative!("static")))
        .manage(Arc::new(Mutex::new(None::<ChorusUser>)))
        .manage(Arc::new(Mutex::new(HashMap::<
            String,
            Vec<HashMap<String, String>>,
        >::new())))
        .attach(rocket::fairing::AdHoc::on_ignite(
            "Observer",
            |rocket| async {
                let user = rocket
                    .state::<Arc<Mutex<Option<ChorusUser>>>>()
                    .unwrap()
                    .clone();
                let app_state = initialize_observer(user).await;
                rocket.manage(app_state)
            },
        ))
}
