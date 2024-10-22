use chorus::types::{Channel, GetChannelMessagesSchema, MessageSendSchema, Snowflake};
use rocket::form::Form;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::tera::Context;
use rocket_dyn_templates::Template;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::request_guards::authencitaced_user::AuthenticatedUser;

#[derive(FromForm)]
pub struct SendMessageForm {
    channel_id: String,
    content: String,
}

#[derive(Deserialize)]
pub struct LoadMoreMessagesRequest {
    channel_id: String,
    last_message_id: Option<String>,
}

#[post("/send_message", data = "<send_message_form>")]
pub async fn send_message(
    send_message_form: Form<SendMessageForm>,
    user: AuthenticatedUser,
    messages_state: &State<Arc<Mutex<HashMap<String, Vec<HashMap<String, String>>>>>>,
) -> Result<Json<HashMap<String, String>>, Custom<Json<HashMap<String, String>>>> {
    let send_message_form = send_message_form.into_inner();
    let mut user_lock = user.0.lock().await;

    if let Some(chorus_user) = user_lock.as_mut() {
        let channel_id = Snowflake::from(send_message_form.channel_id.parse::<u64>().unwrap());

        if let Ok(message) = chorus_user
            .send_message(
                MessageSendSchema {
                    content: Some(send_message_form.content.clone()),
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
                ("id".to_string(), message.id.to_string()),
                (
                    "author".to_string(),
                    chorus_user.object.read().unwrap().username.clone(),
                ),
                ("content".to_string(), send_message_form.content.clone()),
                ("timestamp".to_string(), message.timestamp.to_string()),
                ("edited_timestamp".to_string(), "".to_string()),
            ]));

            let response = HashMap::from([
                ("id".to_string(), message.id.to_string()),
                (
                    "author".to_string(),
                    chorus_user.object.read().unwrap().username.clone(),
                ),
                ("content".to_string(), send_message_form.content),
                ("timestamp".to_string(), message.timestamp.to_string()),
            ]);
            return Ok(Json(response));
        }
    }

    let mut error_response = HashMap::new();
    error_response.insert("error".to_string(), "Failed to send message".to_string());
    Err(Custom(
        rocket::http::Status::InternalServerError,
        Json(error_response),
    ))
}

#[post("/load_more_messages", data = "<load_more_messages_request>")]
pub async fn load_more_messages(
    load_more_messages_request: Json<LoadMoreMessagesRequest>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<HashMap<String, String>>>, Custom<Json<HashMap<String, String>>>> {
    let load_more_messages_request = load_more_messages_request.into_inner();
    let mut user_lock = user.0.lock().await;

    if let Some(chorus_user) = user_lock.as_mut() {
        let channel_id = Snowflake::from(
            load_more_messages_request
                .channel_id
                .parse::<u64>()
                .unwrap(),
        );
        let last_message_id = load_more_messages_request
            .last_message_id
            .map(|id| Snowflake::from(id.parse::<u64>().unwrap()));

        let messages = Channel::messages(
            GetChannelMessagesSchema::before(last_message_id.unwrap_or_else(Snowflake::generate)),
            channel_id,
            chorus_user,
        )
        .await
        .unwrap_or_default();

        let response: Vec<HashMap<String, String>> = messages
            .into_iter()
            .map(|message| {
                HashMap::from([
                    ("id".to_string(), message.id.to_string()),
                    (
                        "author".to_string(),
                        message
                            .author
                            .as_ref()
                            .map_or("Unknown".to_string(), |author| {
                                author
                                    .username
                                    .clone()
                                    .unwrap_or_else(|| "Unknown".to_string())
                            }),
                    ),
                    ("content".to_string(), message.content.unwrap()),
                    ("timestamp".to_string(), message.timestamp.to_string()),
                    (
                        "edited_timestamp".to_string(),
                        message.edited_timestamp.unwrap_or_default().to_string(),
                    ),
                ])
            })
            .collect();

        return Ok(Json(response));
    }

    let mut error_response = HashMap::new();
    error_response.insert(
        "error".to_string(),
        "Failed to load more messages".to_string(),
    );
    Err(Custom(
        rocket::http::Status::InternalServerError,
        Json(error_response),
    ))
}

#[get("/guilds/<guild_id>/<channel_id>")]
pub async fn channel_page(guild_id: &str, channel_id: &str, user: AuthenticatedUser) -> Template {
    let mut context = Context::new();
    context.insert("title", &format!("Channel: {}", channel_id));
    context.insert("guild_id", guild_id);
    context.insert("channel_id", channel_id);

    let mut user_lock = user.0.lock().await;
    let mut guild_data = Vec::new();
    if let Some(chorus_user) = user_lock.as_mut() {
        let guilds = chorus_user.get_guilds(None).await.unwrap_or_default();
        for guild in guilds.iter() {
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

        let channel_id = Snowflake::from(channel_id.parse::<u64>().unwrap());

        // Fetch messages from the channel
        let mut messages = Channel::messages(
            GetChannelMessagesSchema::before(Snowflake::generate()),
            channel_id,
            chorus_user,
        )
        .await
        .unwrap_or_default();
        messages.sort_by_key(|m| m.timestamp); // Sort messages by timestamp
        context.insert("messages", &messages);
        context.insert("guild_data", &guild_data); // Insert guild_data into context
    }

    Template::render("channel", &context.into_json())
}

pub fn routes() -> Vec<Route> {
    routes![channel_page, send_message, load_more_messages]
}
