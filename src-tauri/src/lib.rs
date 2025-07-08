use std::ops::DerefMut;

use coset::{CborSerializable, CoseEncrypt0, CoseEncrypt0Builder, HeaderBuilder};
use secrecy::{zeroize::Zeroize, ExposeSecret, ExposeSecretMut, SecretBox};
use serde::{Deserialize, Serialize};
use sharks::{Sharks, Share};
use tauri::{http::{self, HeaderName, HeaderValue}, State};
use tokio::sync::Mutex;
use tauri_plugin_http::reqwest;
use opaque_ke::{CipherSuite, ClientLogin, ClientLoginFinishParameters, ClientLoginFinishResult, ClientRegistration, ClientRegistrationFinishParameters, CredentialFinalization, CredentialRequest, CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload};
use rand::rngs::OsRng;
use rand::RngCore;
use argon2::Argon2;
use tauri::{Builder, Manager};
use reqwest_middleware::{Middleware, Next, ClientBuilder, Result as MiddlewareResult};
use reqwest::{Request, Response, Client, StatusCode};
use tauri::http::Extensions;
use base64::prelude::*;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit}, aes::Aes256, Aes256Gcm, Key, Nonce
};
use aes_gcm::aead::generic_array::typenum::U12;
use uuid::Uuid;
use sqlx::{sqlite::SqliteConnectOptions,SqlitePool,SqliteTransaction};

fn decrypt_using_nonce(key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, ()> {
    let key = Key::<Aes256Gcm>::from_slice(&key[..32]);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce[..12]);
    cipher.decrypt(nonce,ciphertext).map_err(|_| ())
}

struct DefaultCipherSuite;

impl CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = Argon2<'static>;
}

struct SecureSessionMiddleware {
    session_id: uuid::Uuid,
    session_key: SecretBox<Vec<u8>>,
}

#[async_trait::async_trait]
impl Middleware for SecureSessionMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> MiddlewareResult<Response> {
        req.headers_mut().insert("session-id", self.session_id.to_string().parse().unwrap());
        if let Some(request_body) = req.body_mut() {
            if let Some(request_body_bytes) = request_body.as_bytes() {
                let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
                let cose_protected_header = HeaderBuilder::new().algorithm(coset::iana::Algorithm::A256GCM).build();
                let cose_unprotected_header = HeaderBuilder::new().content_type(String::from("application/cose")).iv(nonce.to_vec()).build();
                let encrypted_body = CoseEncrypt0Builder::new()
                                                                    .protected(cose_protected_header)
                                                                    .unprotected(cose_unprotected_header)
                                                                    .ciphertext(encrypt_with_nonce(self.session_key.expose_secret(), request_body_bytes, nonce).unwrap_or_default())
                                                                    .build();
                *request_body = reqwest::Body::from(encrypted_body.to_vec().unwrap_or_default());
            }
        }
        let res = next.run(req, extensions).await;
        match res {
            Ok(response) => {
                let response_status = response.status();
                let response_bytes = response.bytes().await.unwrap_or_default();
                let response_builder = http::Response::builder().status(response_status);
                let final_response;
                if ! response_bytes.is_empty() {
                    let cose_message = CoseEncrypt0::from_slice(response_bytes.as_ref()).unwrap_or_default();
                    let nonce = cose_message.unprotected.iv;
                    let cbor_encrypted_payload = cose_message.ciphertext.unwrap_or_default();
                    let decrypted_request_content = decrypt_using_nonce(self.session_key.expose_secret(),&cbor_encrypted_payload,&nonce).unwrap_or_default();
                    let decrypted_body = reqwest::Body::from(decrypted_request_content);
                    final_response = response_builder.body(decrypted_body).unwrap();
                } else {
                    final_response = response_builder.body(reqwest::Body::default()).unwrap();
                }
                Ok(Response::from(final_response))
            },
            Err(e) => Err(e),
        }
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
    client_local_data_path: String,
}

#[derive(Serialize,Deserialize)]
struct LoginInitialRequest {
    email: String,
    message: CredentialRequest<DefaultCipherSuite>,
}

#[derive(Serialize,Deserialize)]
struct LoginFinalRequest {
    email: String,
    message: CredentialFinalization<DefaultCipherSuite>,
}

#[derive(Serialize,Deserialize)]
struct UserDataResponse {
    id_user: Uuid,
    email: String,
    name: String,
    server_key_file: Vec<u8>
}

#[derive(Serialize,Deserialize)]
struct NewSecretRequest {
    title: String,
    user_name: Option<String>,
    site: Option<String>,
    notes: Option<String>,
    server_share: String,
}

#[derive(Serialize,Deserialize)]
struct UserRegistrationRequest {
    name: String,
    email: String,
    message: RegistrationRequest<DefaultCipherSuite>
}

#[derive(Serialize,Deserialize)]
struct UserRegistrationFinishResult {
    name: String,
    email: String,
    message: RegistrationUpload<DefaultCipherSuite>
}


#[tauri::command]
async fn register_user(email: String, name: String, master_password: String) -> Result<String, ()> {
    let mut client_rng = OsRng;
    let client_registration_start_result = ClientRegistration::<DefaultCipherSuite>::start(&mut client_rng, master_password.as_bytes()).unwrap();
    let registration_request_bytes = client_registration_start_result.message;
    let mut buffer = Vec::new();
    let registration_request = UserRegistrationRequest {
        name: name.clone(),
        email: email.clone(),
        message: registration_request_bytes.clone(),
    };
    ciborium::ser::into_writer(&registration_request,&mut buffer).map_err(|_| ())?;
    let http_client = reqwest::Client::new();
    let registration_initial_response = http_client.post("http://localhost:3000/auth/user/register")
        .body(buffer)
        .send()
        .await
        .map_err(|_| ())?;
    if registration_initial_response.status() != 200 {
        return Err(());
    }
    let response_bytes = registration_initial_response.bytes().await.map_err(|_| ())?;
    let registration_initial_response: Vec<u8> = ciborium::de::from_reader(response_bytes.as_ref()).map_err(|_| ())?;
    let client_registration_finish_result = client_registration_start_result.state.finish(&mut client_rng, master_password.as_bytes(), RegistrationResponse::deserialize(&registration_initial_response).unwrap(), ClientRegistrationFinishParameters::default()).unwrap();
    let registration_finish_bytes = client_registration_finish_result.message;
    buffer = Vec::new();
    let registration_finish_request = UserRegistrationFinishResult {
        name: name.clone(),
        email: email.clone(),
        message: registration_finish_bytes.clone(),
    };
    ciborium::ser::into_writer(&registration_finish_request,&mut buffer).map_err(|_| ())?;
    let registration_final_response = http_client.post("http://localhost:3000/auth/user/register-finalize")
        .body(buffer)
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
async fn create_client_data(state: State<'_, Mutex<S2SecretData>>) -> Result<String, ()> {
   let state = state.lock().await;
   let client_db_connection_options = SqliteConnectOptions::new()
        .filename(&state.client_local_data_path)
        .create_if_missing(true);
    let client_db_pool = SqlitePool::connect_with(client_db_connection_options).await.map_err(|_| ())?;
    let mut transaction = client_db_pool.begin().await.map_err(|_| ())?;
    sqlx::query("CREATE TABLE IF NOT EXISTS user (id TEXT PRIMARY KEY, client_key_file BLOB NOT NULL)")
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    sqlx::query("CREATE TABLE IF NOT EXISTS secret (id TEXT PRIMARY KEY, client_share BLOB NOT NULL, encryption_key BLOB NOT NULL, FOREIGN KEY (user_id) REFERENCES user(id))")
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    let mut client_key_file = [0u8; 32];
    OsRng.fill_bytes(&mut client_key_file);
    sqlx::query("INSERT INTO user (id, client_key_file) VALUES (?, ?)")
        .bind(state.user_id.unwrap_or_default().to_string())
        .bind(client_key_file.to_vec())
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    Ok("Client data created successfully".to_string())
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
    let response_bytes = user_data.bytes().await.map_err(|_| ())?;
    let user_data_response: UserDataResponse = ciborium::de::from_reader(response_bytes.as_ref()).map_err(|_| ())?;
    state.user_id = Some(user_data_response.id_user);
    state.user_name = Some(user_data_response.name);
    state.user_email = Some(user_data_response.email);
    Ok("Loaded user data".to_string())
}


#[tauri::command]
async fn logout(state: State<'_, Mutex<S2SecretData>>) -> Result<String, ()> {
    let mut state = state.lock().await;
    state.user_id = None;
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
    let mut buffer = Vec::new();
    let login_initial_request = LoginInitialRequest {
        email: email.clone(),
        message: login_request_bytes.clone(),
    };
    ciborium::ser::into_writer(&login_initial_request,&mut buffer).map_err(|_| ())?;
    let login_initial_response = http_client.post("http://localhost:3000/auth/user/login")
    .body(buffer)
        .send()
        .await
        .map_err(|_| ())?;
    if login_initial_response.status() != 200 {
        return Err(());
    }
    let temp_session_id = login_initial_response.headers().get("session-id").unwrap().clone();
    let response_bytes = login_initial_response.bytes().await.map_err(|_| ())?;
    let login_initial_response: Vec<u8> = ciborium::de::from_reader(response_bytes.as_ref()).map_err(|_| ())?;
    let client_login_finish_result = client_login_start_result.state.finish(
        master_password.as_bytes(),
        CredentialResponse::deserialize(&login_initial_response).unwrap(),
        ClientLoginFinishParameters::default(),
    ).unwrap();
    buffer = Vec::new();
    let login_final_request = LoginFinalRequest {
        email: email.clone(),
        message: client_login_finish_result.message.clone(),
    };
    ciborium::ser::into_writer(&login_final_request,&mut buffer).map_err(|_| ())?;
    let client_final_response = http_client.post("http://localhost:3000/auth/user/login-finalize")
        .body(buffer)
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
    .with(SecureSessionMiddleware {
        session_id: uuid::Uuid::parse_str(session_id.to_str().unwrap()).unwrap(),
        session_key: SecretBox::new(Box::new(client_login_finish_result.session_key.to_vec()))
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

fn encrypt_with_nonce(key: &[u8], plaintext: &[u8], nonce: Nonce<U12>) -> Result<Vec<u8>, ()> {
    let key = Key::<Aes256Gcm>::from_slice(&key[..32]);
    let cipher = Aes256Gcm::new(&key);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).map_err(|_| ())?;
    Ok(ciphertext)
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
    let mut buffer = Vec::new();
    let new_secret_request = NewSecretRequest {
        title: BASE64_STANDARD.encode(encrypted_title),
        user_name: if user_name.is_empty() { None } else { Some(BASE64_STANDARD.encode(encrypted_user_name)) },
        site: if site.is_empty() { None } else { Some(BASE64_STANDARD.encode(encrypted_site)) },
        notes: if notes.is_empty() { None } else { Some(BASE64_STANDARD.encode(encrypted_notes)) },
        server_share: BASE64_STANDARD.encode(Vec::from(&shares[0])),
    };
    ciborium::ser::into_writer(&new_secret_request,&mut buffer).map_err(|_| ())?;
    let add_secret_response = state.http_client.post("http://localhost:3000/secrets")
        .body(buffer)
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
