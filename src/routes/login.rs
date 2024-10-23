use chorus::errors::ChorusError;
use chorus::instance::{ChorusUser, Instance};
use chorus::types::LoginSchema;
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::tokio::sync::Mutex;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::tera::Context;
use rocket_dyn_templates::Template;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(FromForm)]
pub struct LoginForm {
    instance_url: String,
    username: String,
    password: String,
    #[field(name = "h-captcha-response")]
    captcha_response: Option<String>,
}

#[post("/login", data = "<login_form>")]
pub async fn login(
    login_form: Form<LoginForm>,
    user: &State<Arc<Mutex<Option<ChorusUser>>>>,
) -> Result<Redirect, Template> {
    let instance_result = Instance::new(&login_form.instance_url, None).await;

    let mut context = Context::new();
    context.insert("instance_url", &login_form.instance_url);
    context.insert("username", &login_form.username);
    context.insert("password", &login_form.password);
    context.insert("authenticated", &"false".to_string());
    context.insert("users", &Vec::<String>::new()); // Ensure users is always present

    match instance_result {
        Ok(mut instance) => {
            let login_schema = LoginSchema {
                login: login_form.username.clone(),
                password: login_form.password.clone(),
                captcha_key: login_form.captcha_response.clone(),
                ..Default::default()
            };

            let user_result = instance.login_account(login_schema).await;
            match user_result {
                Ok(logged_in_user) => {
                    let mut user_lock = user.lock().await;
                    *user_lock = Some(logged_in_user.clone());

                    let username = logged_in_user.object.read().unwrap().username.clone();
                    context.insert("authenticated", &"true".to_string());
                    context.insert("user", &username);

                    // Get the list of users
                    let users: Vec<String> = vec![username.clone()];
                    context.insert("users", &users);

                    return Ok(Redirect::to(uri!("/home")));
                }
                Err(ChorusError::ReceivedErrorCode { error_code, error }) => {
                    handle_login_error(&mut context, error_code, error);
                }
                Err(e) => {
                    println!("Login failed: {}", e);
                    context.insert("error", &format!("Login failed: {}", e));
                }
            }
        }
        Err(e) => {
            println!("Failed to connect to the Spacebar server: {}", e);
            context.insert(
                "error",
                &format!("Failed to connect to the Spacebar server: {}", e),
            );
        }
    }
    Err(Template::render("index", &context.into_json()))
}

fn handle_login_error(context: &mut Context, error_code: u16, error: String) {
    let error_message = format!("{}", error);
    println!("Login failed: {} - {}", error_code, error_message);
    if let Ok(error_response) = serde_json::from_str::<HashMap<String, Value>>(&error_message) {
        if let Some(captcha_required) = error_response.get("captcha_key") {
            if captcha_required.as_array().map_or(false, |arr| {
                arr.contains(&Value::String("captcha-required".to_string()))
            }) {
                context.insert("captcha_required", &"true".to_string());
                if let Some(sitekey) = error_response.get("captcha_sitekey") {
                    context.insert("captcha_sitekey", &sitekey.as_str().unwrap().to_string());
                }
            }
        }
    }
    context.insert(
        "error",
        &format!("Login failed: {} - {}", error_code, error_message),
    );
}

pub fn routes() -> Vec<Route> {
    routes![login]
}
