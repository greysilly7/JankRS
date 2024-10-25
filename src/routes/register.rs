use chorus::errors::ChorusError;
use chorus::instance::{ChorusUser, Instance};
use chorus::types::RegisterSchema;
use chrono::NaiveDate;
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
pub struct RegisterForm {
    instance_url: String,
    username: String,
    password: String,
    email: Option<String>,
    date_of_birth: Option<String>,
    #[field(name = "h-captcha-response")]
    captcha_response: Option<String>,
}

#[get("/register")]
pub async fn register_page() -> Template {
    let mut context = Context::new();
    context.insert("instance_url", &"".to_string());
    context.insert("username", &"".to_string());
    context.insert("password", &"".to_string());
    context.insert("email", &"".to_string());
    context.insert("date_of_birth", &"".to_string());
    Template::render("register", &context.into_json())
}

#[post("/register", data = "<register_form>")]
pub async fn register(
    register_form: Form<RegisterForm>,
    user: &State<Arc<Mutex<Option<ChorusUser>>>>,
) -> Result<Redirect, Template> {
    let mut context = Context::new();
    context.insert("instance_url", &register_form.instance_url);
    context.insert("username", &register_form.username);
    context.insert("password", &register_form.password);
    context.insert("email", &register_form.email.clone().unwrap_or_default()); // Add email to context
    context.insert("date_of_birth", &register_form.date_of_birth.clone().unwrap_or_default()); // Add date_of_birth to context
    context.insert("authenticated", &"false".to_string());
    context.insert("users", &Vec::<String>::new()); // Ensure users is always present

    println!("{:?}", &register_form.instance_url);
    
    let instance_result = Instance::new(&register_form.instance_url, None).await;

    match instance_result {
        Ok(mut instance) => {
          println!("{:?}", &instance);
            let date_of_birth = register_form.date_of_birth.as_ref()
                .and_then(|dob| NaiveDate::parse_from_str(dob, "%Y-%m-%d").ok());

            let register_schema = RegisterSchema {
                username: register_form.username.clone(),
                password: Some(register_form.password.clone()),
                email: register_form.email.clone(),
                date_of_birth,
                captcha_key: register_form.captcha_response.clone(),
                consent: true, // Assuming consent is always true for registration
                ..Default::default()
            };

            let user_result = instance.register_account(register_schema).await;
            match user_result {
                Ok(registered_user) => {
                    let mut user_lock = user.lock().await;
                    *user_lock = Some(registered_user.clone());

                    let username = registered_user.object.read().unwrap().username.clone();
                    context.insert("authenticated", &"true".to_string());
                    context.insert("user", &username);

                    // Get the list of users
                    let users: Vec<String> = vec![username.clone()];
                    context.insert("users", &users);

                    return Ok(Redirect::to(uri!("/")));
                }
                Err(ChorusError::ReceivedErrorCode { error_code, error }) => {
                    handle_register_error(&mut context, error_code, error);
                }
                Err(e) => {
                    println!("Registration failed: {}", e);
                    context.insert("register_error", &format!("Registration failed: {}", e));
                }
            }
        }
        Err(e) => {
            println!("Failed to connect to the Spacebar server: {}", e);
            context.insert(
                "register_error",
                &format!("Failed to connect to the Spacebar server: {}", e),
            );
        }
    }
    Err(Template::render("register", &context.into_json()))
}

fn handle_register_error(context: &mut Context, error_code: u16, error: String) {
    let error_message = format!("{}", error);
    println!("Registration failed: {} - {}", error_code, error_message);
    if let Ok(error_response) = serde_json::from_str::<HashMap<String, Value>>(&error_message) {
        if let Some(captcha_required) = error_response.get("captcha_key") {
            if captcha_required.as_array().map_or(false, |arr| {
                arr.contains(&Value::String("captcha-required".to_string()))
            }) {
                context.insert("captcha_required", &"true".to_string());
                if let Some(sitekey) = error_response.get("captcha_sitekey") {
                    context.insert("captcha_sitekey", &sitekey.as_str().unwrap().to_string());
                }
                if let Some(captcha_type) = error_response.get("captcha_service") {
                  println!("{}", serde_json::to_string(&error_response).unwrap());
                    context.insert("captcha_service", &captcha_type.as_str().unwrap().to_string());
                }
            }
        }
    }
    context.insert(
        "register_error",
        &format!("Registration failed: {} - {}", error_code, error_message),
    );
}

pub fn routes() -> Vec<Route> {
    routes![register_page, register]
}