use chorus::instance::ChorusUser;
use rocket::tokio::sync::Mutex;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::tera::Context;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::sync::Arc;

#[get("/home")]
pub async fn home(user: &State<Arc<Mutex<HashMap<String, ChorusUser>>>>) -> Template {
    let mut user_lock = user.lock().await;
    let mut context = Context::new();

    let mut users_data = Vec::new();

    for (instance_url, chorus_user) in user_lock.iter_mut() {
        let username = chorus_user.object.read().unwrap().username.clone();

        // Fetch guilds
        let guilds = chorus_user.get_guilds(None).await.unwrap_or_default();
        let guild_names: Vec<String> = guilds.iter().filter_map(|g| g.name.clone()).collect();

        // Fetch private messages (assuming a method `get_private_channels` exists)
        let private_channels = chorus_user.get_private_channels().await.unwrap_or_default();
        let private_message_contents: Vec<String> = private_channels
            .iter()
            .filter_map(|c| c.name.clone())
            .collect();

        users_data.push(serde_json::json!({
            "instance_url": instance_url,
            "username": username,
            "guilds": guild_names,
            "private_messages": private_message_contents,
        }));
    }

    context.insert("title", "Home");
    context.insert("users_data", &users_data);
    Template::render("home", &context.into_json())
}

pub fn routes() -> Vec<Route> {
    routes![home]
}
