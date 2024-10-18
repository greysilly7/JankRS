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
    user: &State<Arc<Mutex<HashMap<String, ChorusUser>>>>,
) -> Result<Redirect, Template> {
    let send_message_form = send_message_form.into_inner();
    let mut user_lock = user.lock().await;

    for (_, chorus_user) in user_lock.iter_mut() {
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
    user: &State<Arc<Mutex<HashMap<String, ChorusUser>>>>,
) -> Template {
    let mut context = Context::new();
    context.insert("title", &format!("Channel: {}", channel_id));
    context.insert("guild_id", guild_id);
    context.insert("channel_id", channel_id);

    let mut user_lock = user.lock().await;
    let mut channel_data = Vec::new();
    for (instance_url, chorus_user) in user_lock.iter_mut() {
        let channel_id = Snowflake::from(channel_id.parse::<u64>().unwrap());
        let messages = Channel::messages(
            GetChannelMessagesSchema::before(Snowflake::generate()),
            channel_id,
            chorus_user,
        )
        .await;
        if let Ok(messages) = messages {
            let messages: Vec<HashMap<String, String>> = messages
                .iter()
                .map(|m| {
                    let author = m
                        .author
                        .as_ref()
                        .map_or("Unknown".to_string(), |a| a.username.clone().unwrap());
                    let content = m.content.clone().unwrap_or_default();
                    let mut message_data = HashMap::new();
                    message_data.insert("author".to_string(), author);
                    message_data.insert("content".to_string(), content);
                    message_data
                })
                .collect();
            channel_data.push(serde_json::json!({
                "channel_id": channel_id.to_string(),
                "messages": messages,
            }));
        }
    }
    context.insert("channel_data", &channel_data);

    Template::render("channel", &context.into_json())
}

pub fn routes() -> Vec<Route> {
    routes![channel_page, send_message]
}
