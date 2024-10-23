use rocket::Route;
use rocket_dyn_templates::tera::Context;
use rocket_dyn_templates::Template;

use crate::request_guards::authencitaced_user::AuthenticatedUser;

#[get("/guilds/<guild_id>")]
pub async fn guild_page(guild_id: &str, user: AuthenticatedUser) -> Template {
    let mut context = Context::new();
    context.insert("title", &format!("Guild: {}", guild_id));
    context.insert("guild_id", guild_id);

    let mut user_lock = user.0.lock().await;
    let mut guild_data = Vec::new();
    if let Some(chorus_user) = user_lock.as_mut() {
        let guilds = chorus_user.get_guilds(None).await.unwrap_or_default();
        for guild in guilds.iter() {
            let channels = match guild.channels(chorus_user).await {
                Ok(channels) => channels,
                Err(_) => continue, // Skip this guild if channels cannot be fetched
            };
            let mut channels_data = Vec::new();
            for channel in channels {
                channels_data.push(serde_json::json!({
                    "channel_name": channel.name.clone().unwrap_or_default(),
                    "channel_id": channel.id.to_string(),
                }));
            }
            guild_data.push(serde_json::json!({
                "guild_id": guild.id.to_string(),
                "guild_name": guild.name.clone().unwrap_or_default(),
                "guild_icon": guild.icon.clone().unwrap_or_default(),
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
