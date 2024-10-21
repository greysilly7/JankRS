use chorus::instance::ChorusUser;
use chorus::types::{Channel, GetChannelMessagesSchema, MessageSendSchema, Snowflake};
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::tokio::sync::Mutex;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::tera::Context;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(FromForm)]
pub struct SendMessageForm {
    guild_id: String,
    channel_id: String,
    content: String,
}

#[post("/send_message", data = "<send_message_form>")]
pub async fn send_message(
    send_message_form: Form<SendMessageForm>,
    user: &State<Arc<Mutex<Option<ChorusUser>>>>,
    messages_state: &State<Arc<Mutex<HashMap<String, Vec<HashMap<String, String>>>>>>,
) -> Result<Redirect, Template> {
    let send_message_form = send_message_form.into_inner();
    let mut user_lock = user.lock().await;

    if let Some(chorus_user) = user_lock.as_mut() {
        let channel_id = Snowflake::from(send_message_form.channel_id.parse::<u64>().unwrap());

        if let Ok(_) = chorus_user
            .send_message(
                MessageSendSchema {
                    content: Some(send_message_form.content.to_string()),
                    ..Default::default()
                },
                channel_id,
            )
            .await
        {
            // Update the messages state
            let mut messages_lock = messages_state.lock().await;
            let messages = messages_lock
                .entry(send_message_form.channel_id.clone())
                .or_default();
            messages.push(HashMap::from([
                (
                    "author".to_string(),
                    chorus_user.object.read().unwrap().username.clone(),
                ),
                ("content".to_string(), send_message_form.content),
                ("edited_timestamp".to_string(), "".to_string()),
            ]));

            return Ok(Redirect::to(uri!(channel_page(
                guild_id = send_message_form.guild_id,
                channel_id = send_message_form.channel_id
            ))));
        }
    }

    let mut context = Context::new();
    context.insert("error", "Failed to send message");
    Err(Template::render("error", &context.into_json()))
}

#[get("/guilds/<guild_id>/<channel_id>")]
pub async fn channel_page(
    guild_id: &str,
    channel_id: &str,
    user: &State<Arc<Mutex<Option<ChorusUser>>>>,
) -> Template {
    let mut context = Context::new();
    context.insert("title", &format!("Channel: {}", channel_id));
    context.insert("guild_id", guild_id);
    context.insert("channel_id", channel_id);

    let mut user_lock = user.lock().await;
    let mut channel_data = HashMap::new();
    let mut guild_data = Vec::new();
    if let Some(chorus_user) = user_lock.as_mut() {
        let channel_id = Snowflake::from(channel_id.parse::<u64>().unwrap());

        // Fetch messages from the channel
        let messages = Channel::messages(
            GetChannelMessagesSchema::before(Snowflake::generate()),
            channel_id,
            chorus_user,
        )
        .await;
        if let Ok(mut messages) = messages {
            messages.sort_by_key(|m| m.timestamp); // Sort messages by timestamp
            channel_data.insert("messages".to_string(), messages);
        }

        let guilds = chorus_user.get_guilds(None).await.unwrap_or_default();
        for guild in guilds {
            let channels = guild.channels(chorus_user).await.unwrap();
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
    context.insert("channel_data", &channel_data);
    context.insert("guild_data", &guild_data);

    Template::render("channel", &context.into_json())
}

pub fn routes() -> Vec<Route> {
    routes![channel_page, send_message]
}
