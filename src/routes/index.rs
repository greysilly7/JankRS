use chorus::instance::ChorusUser;
use rocket::tokio::sync::Mutex;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::sync::Arc;

#[get("/")]
pub async fn index(user: &State<Arc<Mutex<Option<ChorusUser>>>>) -> Template {
    let mut context = HashMap::from([
        ("instance_url", "".to_string()),
        ("username", "".to_string()),
        ("password", "".to_string()),
        ("authenticated", "false".to_string()),
    ]);

    let user = user.lock().await;
    if let Some(user) = &*user {
        println!("authenticated: {:?}", context.get("authenticated"));
        let username = user.object.read().unwrap().username.clone();
        context.insert("authenticated", "true".to_string());
        context.insert("user", username);
    }

    Template::render("index", &context)
}

pub fn routes() -> Vec<Route> {
    routes![index]
}
