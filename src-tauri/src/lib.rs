use secrecy::{zeroize::Zeroize, ExposeSecret, ExposeSecretMut, SecretBox};
use sharks::{Sharks, Share};
use tauri::State;
use tokio::sync::Mutex;
use tauri_plugin_http::reqwest;
use opaque_ke::{CipherSuite, ClientLogin, ClientLoginFinishParameters, ClientRegistration, ClientRegistrationFinishParameters, CredentialResponse, RegistrationResponse};
use rand::rngs::OsRng;
use argon2::Argon2;
use tauri::{Builder, Manager};
use reqwest_middleware::{Middleware, Next, ClientBuilder, Result as MiddlewareResult};
use reqwest::{Request, Response, Client};
use tauri::http::Extensions;
use base64::prelude::*;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit}, aes::Aes256, Aes256Gcm, Key, Nonce
};
use uuid::Uuid;

struct DefaultCipherSuite;

impl CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = Argon2<'static>;
}

struct SessionHeaderMiddleware {
    /// Shared state holding the current session id, if any.
    session_id: Option<uuid::Uuid>,
}

#[async_trait::async_trait]
impl Middleware for SessionHeaderMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> MiddlewareResult<Response> {
        if let Some(session_id) = &self.session_id {
            req.headers_mut().insert("session-id", session_id.to_string().parse().unwrap());
        }
        let res = next.run(req, extensions).await;
        res
    }
}


#[derive(Default)]
struct S2SecretData {
    user_id: Option<Uuid>,
    user_name: Option<String>,
    user_email: Option<String>,
    session_id: Option<uuid::Uuid>,
    session_key: SecretBox<Option<Vec<u8>>>,
    password_encryption_key: SecretBox<Option<Vec<u8>>>,
    server_key_file: SecretBox<Option<Vec<u8>>>,
    http_client: reqwest_middleware::ClientWithMiddleware,
}


#[tauri::command]
async fn register_user(email: String, name: String, master_password: String) -> Result<String, ()> {
    let mut client_rng = OsRng;
    let client_registration_start_result = ClientRegistration::<DefaultCipherSuite>::start(&mut client_rng, master_password.as_bytes()).unwrap();
    let registration_request_bytes = client_registration_start_result.message;

    let http_client = reqwest::Client::new();
    let registration_initial_response = http_client.post("http://localhost:3000/auth/user/register")
        .json(&serde_json::json!({
            "name": name,
            "email": email,
            "message": registration_request_bytes
        }))
        .send()
        .await
        .map_err(|_| ())?;
    if registration_initial_response.status() != 200 {
        return Err(());
    }
    let registration_initial_response_json: Vec<u8> = registration_initial_response.json().await.map_err(|_| ())?;
    let client_registration_finish_result = client_registration_start_result.state.finish(&mut client_rng, master_password.as_bytes(), RegistrationResponse::deserialize(&registration_initial_response_json).unwrap(), ClientRegistrationFinishParameters::default()).unwrap();
    let registration_finish_bytes = client_registration_finish_result.message;
    let registration_final_response = http_client.post("http://localhost:3000/auth/user/register-finalize")
        .json(&serde_json::json!({
            "name": name,
            "email": email,
            "message": registration_finish_bytes
        }))
        .send()
        .await
        .map_err(|_| ())?;
    if registration_final_response.status() != 200 {
        return Err(());
    }
    Ok("User registered successfully".to_string())
}

#[tauri::command]
async fn is_authenticated(state: State<'_, Mutex<S2SecretData>>) -> Result<bool, ()> {
    let state = state.lock().await;
    Ok(state.session_id.is_some())
}

#[tauri::command]
async fn logged_user_data(state: State<'_, Mutex<S2SecretData>>) -> Result<String, ()> {
    let mut state = state.lock().await;
    let user_data = state.http_client.get("http://localhost:3000/user")
        .send()
        .await
        .map_err(|_| ())?;
    if user_data.status() != 200 {
        return Err(());
    }
    let user_data_json: serde_json::Value = user_data.json().await.map_err(|_| ())?;
    state.user_id = Some(Uuid::parse_str(user_data_json["id_user"].as_str().unwrap()).unwrap());
    state.user_name = Some(user_data_json["name"].as_str().unwrap().to_string());
    state.user_email = Some(user_data_json["email"].as_str().unwrap().to_string());
    Ok("Loaded user data".to_string())
}


#[tauri::command]
async fn logout(state: State<'_, Mutex<S2SecretData>>) -> Result<String, ()> {
    let mut state = state.lock().await;
    state.user_name = None;
    state.user_email = None;
    state.session_key.zeroize();
    state.password_encryption_key.zeroize();
    state.server_key_file.zeroize();
    state.http_client.post("http://localhost:3000/auth/user/logout")
    .send()
    .await
    .map_err(|_| ())?;
    state.session_id = None;
    //let http_client = reqwest::Client::new();
    //#if let Some(session_id) = state.session_id {
    //    http_client.post("http://localhost:3000/auth/user/logout")
    //    .header("session-id", &session_id.to_string())
    //    .send()
    //    .await
    //    .map_err(|_| ())?;
    //    state.session_id = None;
    //}
    Ok("User logged out successfully".to_string())
}

#[tauri::command]
async fn login(state: State<'_, Mutex<S2SecretData>>, email: String, master_password: String) -> Result<String, ()> {
    let mut client_rng = OsRng;
    let client_login_start_result = ClientLogin::<DefaultCipherSuite>::start(&mut client_rng, master_password.as_bytes()).unwrap();
    let login_request_bytes = client_login_start_result.message;
    let http_client = reqwest::Client::new();
    let login_initial_response = http_client.post("http://localhost:3000/auth/user/login")
        .json(&serde_json::json!({
            "email": email,
            "message": login_request_bytes
        }))
        .send()
        .await
        .map_err(|_| ())?;
    if login_initial_response.status() != 200 {
        return Err(());
    }
    let temp_session_id = login_initial_response.headers().get("session-id").unwrap().clone();
    let login_initial_response_json: Vec<u8> = login_initial_response.json().await.map_err(|_| ())?;
    let client_login_finish_result = client_login_start_result.state.finish(
        master_password.as_bytes(),
        CredentialResponse::deserialize(&login_initial_response_json).unwrap(),
        ClientLoginFinishParameters::default(),
    ).unwrap();
    let client_final_response = http_client.post("http://localhost:3000/auth/user/login-finalize")
        .json(&serde_json::json!({
            "email": email,
            "message": client_login_finish_result.message
        }))
        .header("session-id", &temp_session_id)
        .send()
        .await
        .map_err(|_| ())?;
    if client_final_response.status() != 200 {
        return Err(());
    }
    let session_id = client_final_response.headers().get("session-id").unwrap().clone();
    let mut state = state.lock().await;
    state.session_id = Some(uuid::Uuid::parse_str(session_id.to_str().unwrap()).unwrap());
    state.session_key = SecretBox::new(Box::new(Some(client_login_finish_result.session_key.to_vec())));
    state.password_encryption_key = SecretBox::new(Box::new(Some(client_login_finish_result.export_key.to_vec())));
    let http_client = Client::builder().build().unwrap();
    state.http_client = ClientBuilder::new(http_client)
    .with(SessionHeaderMiddleware {
        session_id: state.session_id.clone(),
    })
    .build();
    Ok("User login successfully".to_string())
}

fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, ()> {
    let key = Key::<Aes256Gcm>::from_slice(&key[..32]);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).map_err(|_| ())?;
    Ok([nonce.to_vec(), ciphertext].concat())
}

fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    let key = Key::<Aes256Gcm>::from_slice(&key[..32]);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    cipher.decrypt(nonce, &ciphertext[12..]).map_err(|_| ())
}

#[tauri::command]
async fn delete_secret(state: State<'_, Mutex<S2SecretData>>, secret_id: Uuid) -> Result<String, ()> {
    let state = state.lock().await;
    let delete_secret_response = state.http_client.delete(format!("http://localhost:3000/secrets/{}", &secret_id))
        .send()
        .await
        .map_err(|_| ())?;
    if delete_secret_response.status() != 200 {
        return Err(());
    }
    Ok("Secret deleted successfully".to_string())
}

#[tauri::command]
async fn add_secret(state: State<'_, Mutex<S2SecretData>>, title: String, user_name: String, password: String, site: String, notes: String) -> Result<String, ()> {
    let sharks = Sharks(2);
    let dealer = sharks.dealer(password.as_bytes());
    let shares: Vec<Share> = dealer.take(2).collect();
    let state = state.lock().await;
    let password_key = state.password_encryption_key.expose_secret().as_ref().unwrap();
    
    let encrypted_title = encrypt(password_key, title.as_bytes())?;
    let encrypted_user_name = encrypt(password_key, user_name.as_bytes())?;
    let encrypted_site =  encrypt(password_key, site.as_bytes())?;
    let encrypted_notes =  encrypt(password_key, notes.as_bytes())?;
    let add_secret_response = state.http_client.post("http://localhost:3000/secrets")
        .json(&serde_json::json!({
            "title": BASE64_STANDARD.encode(encrypted_title),
            "user_name": BASE64_STANDARD.encode(encrypted_user_name),
            "server_share": BASE64_STANDARD.encode(Vec::from(&shares[0])),
            "site": BASE64_STANDARD.encode(encrypted_site),
            "notes": BASE64_STANDARD.encode(encrypted_notes)
        }))
        .send()
        .await
        .map_err(|_| ())?;
    if add_secret_response.status() != 200 {
        return Err(());
    }
    Ok("Secret added successfully".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(S2SecretData::default()));
            Ok(())
        })
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![login,register_user,is_authenticated,logout, add_secret, delete_secret, logged_user_data])
        .run(tauri::generate_context!())
        .expect("error while running S2Secret application");
}
