use chorus::instance::ChorusUser;
use rocket::tokio::sync::Mutex;
use rocket::State;
use rocket::{get, Route};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::sync::Arc;

#[get("/")]
pub async fn index(user: &State<Arc<Mutex<Option<ChorusUser>>>>) -> Template {
    let mut context = create_initial_context();

    let user = user.lock().await;
    if let Some(user) = &*user {
        handle_authenticated_user(&mut context, user);
    } else {
        // Ensure users is always present
        context.insert(
            "users",
            serde_json::to_string(&Vec::<String>::new()).unwrap(),
        );
    }

    Template::render("index", &context)
}

fn create_initial_context() -> HashMap<&'static str, String> {
    HashMap::from([
        ("instance_url", "".to_string()),
        ("username", "".to_string()),
        ("password", "".to_string()),
        ("authenticated", "false".to_string()),
        ("email", "".to_string()),
    ])
}

fn handle_authenticated_user(context: &mut HashMap<&'static str, String>, user: &ChorusUser) {
    println!("authenticated: {:?}", context.get("authenticated"));
    let username = user.object.read().unwrap().username.clone();
    context.insert("authenticated", "true".to_string());
    context.insert("user", username.clone());

    // Add the user to the users list
    let users: Vec<String> = vec![username];
    context.insert("users", serde_json::to_string(&users).unwrap());
}

pub fn routes() -> Vec<Route> {
    routes![index]
}