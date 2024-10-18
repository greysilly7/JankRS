use chorus::instance::ChorusUser;
use chorus::types::{Channel, GetChannelMessagesSchema, Snowflake};
use rocket::tokio::sync::Mutex;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::tera::Context;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::sync::Arc;

#[get("/guilds/<guild_id>")]
pub async fn guild_page(
    guild_id: &str,
    user: &State<Arc<Mutex<HashMap<String, ChorusUser>>>>,
) -> Template {
    let mut context = Context::new();
    context.insert("title", &format!("Guild: {}", guild_id));
    context.insert("guild_id", guild_id);

    let mut user_lock = user.lock().await;
    let mut guild_data = Vec::new();
    for (instance_url, chorus_user) in user_lock.iter_mut() {
        let guilds = chorus_user.get_guilds(None).await.unwrap_or_default();
        let guild = guilds.iter().find(|g| g.id.to_string() == guild_id);
        if let Some(guild) = guild {
            let channels = guild.channels(chorus_user).await.unwrap();
            let mut channels_data = Vec::new();
            for channel in channels {
                channels_data.push(serde_json::json!({
                    "channel_name": channel.name.clone().unwrap_or_default(),
                    "channel_id": channel.id.to_string(),
                }));
            }
            guild_data.push(serde_json::json!({
                "instance_url": instance_url,
                "guild": guild,
                "channels": channels_data,
            }));
        }
    }
    context.insert("guild_data", &guild_data);

    Template::render("guild", &context.into_json())
}

pub fn routes() -> Vec<Route> {
    routes![guild_page]
}
