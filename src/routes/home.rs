use chorus::instance::ChorusUser;
use rocket::tokio::sync::Mutex;
use rocket::State;
use rocket::{get, Route};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::sync::Arc;

#[get("/home")]
pub async fn home(user: &State<Arc<Mutex<Option<ChorusUser>>>>) -> Template {
    let mut context = HashMap::new();
    let mut users_data = Vec::new();

    let user = user.lock().await;
    if let Some(chorus_user) = &*user {
        let username = chorus_user.object.read().unwrap().username.clone();
        let guilds_data = fetch_guilds_data(chorus_user.clone()).await;
        let private_message_contents = fetch_private_messages(chorus_user.clone()).await;

        users_data.push(serde_json::json!({
            "username": username,
            "guilds": guilds_data,
            "private_messages": private_message_contents,
        }));
    }

    context.insert("users_data", serde_json::json!(users_data));
    Template::render("home", &context)
}

async fn fetch_guilds_data(mut chorus_user: ChorusUser) -> Vec<HashMap<String, String>> {
    let guilds = chorus_user.get_guilds(None).await.unwrap_or_default();
    guilds
        .iter()
        .map(|g| {
            let mut guild_data = HashMap::new();
            guild_data.insert("guild_id".to_string(), g.id.to_string());
            guild_data.insert("guild_name".to_string(), g.name.clone().unwrap_or_default());
            guild_data.insert("guild_icon".to_string(), g.icon.clone().unwrap_or_default());
            guild_data
        })
        .collect()
}

async fn fetch_private_messages(mut chorus_user: ChorusUser) -> Vec<String> {
    let private_channels = chorus_user.get_private_channels().await.unwrap_or_default();
    private_channels
        .iter()
        .filter_map(|c| c.name.clone())
        .collect()
}
pub fn routes() -> Vec<Route> {
    routes![home]
}
