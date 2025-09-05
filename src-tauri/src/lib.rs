use std::{collections::HashMap, ops::DerefMut, path::Path};

use coset::{CborSerializable, CoseEncrypt0, CoseEncrypt0Builder, HeaderBuilder};
use secrecy::{zeroize::Zeroize, ExposeSecret, ExposeSecretMut, SecretBox};
use serde::{Deserialize, Serialize};
use sharks::{Sharks, Share};
use tauri::{http::{self, HeaderName, HeaderValue}, State};
use tauri_plugin_shell::open::Program;
use tokio::sync::Mutex;
use tauri_plugin_http::reqwest;
use opaque_ke::{CipherSuite, ClientLogin, ClientLoginFinishParameters, ClientLoginFinishResult, ClientRegistration, ClientRegistrationFinishParameters, CredentialFinalization, CredentialRequest, CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload};
use rand::{rngs::OsRng, Rng};
use rand::RngCore;
use argon2::Argon2;
use argon2::password_hash::{SaltString, PasswordHasher};
use tauri::{Builder, Manager};
use reqwest_middleware::{Middleware, Next, ClientBuilder, Result as MiddlewareResult};
use reqwest::{Request, Response, Client, StatusCode};
use tauri::http::Extensions;
use base64::prelude::*;
use aes_gcm::{
    aead::{consts::U800, Aead, AeadCore, KeyInit}, aes::Aes256, Aes256Gcm, Key, Nonce
};
use aes_gcm::aead::generic_array::typenum::U12;
use uuid::Uuid;
use sqlx::{sqlite::{SqliteConnectOptions, UpdateHookResult},SqlitePool,SqliteTransaction};
use chrono::{NaiveDateTime, Utc};
use sqlx::Row;
use tauri_plugin_clipboard_manager::ClipboardExt;
use hmac_sha512::HMAC;
use bincode::{config, enc, Decode, Encode};
use tauri_plugin_dialog::DialogExt;


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

#[derive(Deserialize, Serialize)]
pub struct SecretShare {
    server_share: Vec<u8>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Serialize)]
struct EmergencyAccessRequest {
    id_emergency_contact: Uuid,
    server_ticket: Vec<u8>,
    server_v: Vec<u8>,
    server_a: Vec<u8>
}

#[derive(Serialize,Deserialize)]
struct S2SecretUserUpsertResponse {
    id_user: Uuid
}

#[derive(Deserialize, Serialize, Encode, Decode)]
pub struct Ticket {
    pub password_hash: String,
    pub encrypted_secret: Vec<u8>,
}

struct SecureSessionMiddleware {
    session_id: uuid::Uuid,
    session_key: SecretBox<Vec<u8>>,
}
#[derive(Clone,Serialize,Deserialize)]
struct Password {
    id: Uuid,
    title: String,
    user_name: Option<String>,
    site: Option<String>,
    notes: Option<String>,
    password: Option<String>, // TODO: Remove this field, it should not be stored in the passwords map
    share_updated_at: NaiveDateTime,
    next_share_update: Option<NaiveDateTime>,
    proactive_protection: Option<String>,
}

#[derive(Deserialize, Serialize)]
enum ProactiveProtection {
    Medium,
    High,
    Extreme
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
    passwords: HashMap<Uuid, Password>,
    emergency_contacts: HashMap<Uuid, EmergencyContact>,
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
struct UpsertSecretResponse {
    id_secret: uuid::Uuid,
}

#[derive(Serialize,Deserialize)]
struct EmergencyContactUpsertResponse {
    id_emergency_contact: Uuid
}

#[derive(Serialize,Deserialize)]
struct SecretUpsertRequest {
    title: Vec<u8>,
    user_name: Option<Vec<u8>>,
    site: Option<Vec<u8>>,
    notes: Option<Vec<u8>>,
    server_share: Vec<u8>,
}


#[derive(Deserialize, Serialize)]
struct OneTimeSecretCodeRequest {
    email: String,
    secret_code: String,
}

#[derive(Deserialize, Serialize)]
pub struct EmergencyContactRequest {
    email: String,
    description: Option<String>,
    server_key_file : Vec<u8>,
    server_share: Vec<u8>,
}

#[derive(Clone,Deserialize, Serialize)]
pub struct EmergencyContact {
    id_emergency_contact: Uuid,
    email: String,
    description: Option<String>,
    server_key_file : Vec<u8>,
    server_share: Vec<u8>,
}



#[derive(Deserialize, Serialize)]
pub struct ShareRenewal {
    share: Vec<u8>,
    updated_at: NaiveDateTime
}

#[derive(Deserialize, Serialize)]
struct SecretPatchRequest {
    title: Option<Vec<u8>>,
    user_name: Option<Vec<u8>>,
    site: Option<Vec<u8>>,
    notes: Option<Vec<u8>>,
    server_share: Option<Vec<u8>>,
}

#[derive(Serialize,Deserialize)]
struct Secret {
    id_secret: Uuid,
    title: Vec<u8>,
    user_name: Option<Vec<u8>>,
    site: Option<Vec<u8>>,
    notes: Option<Vec<u8>>,
    share_updated_at: NaiveDateTime,
    next_share_update: Option<NaiveDateTime>,
    proactive_protection: Option<String>,
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
async fn register_user(state: State<'_, Mutex<S2SecretData>>,app_handle: tauri::AppHandle,email: String, name: String, master_password: String) -> Result<String, ()> {
    let mut state = state.lock().await;
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
    if registration_final_response.status() != 201 {
        return Err(());
    } else {
        let registration_final_response_bytes = registration_final_response.bytes().await.map_err(|_| ())?;
        let new_user_id_response: S2SecretUserUpsertResponse = ciborium::de::from_reader(registration_final_response_bytes.as_ref()).map_err(|_| ())?;
        create_client_data(&mut state, &app_handle, &new_user_id_response.id_user).await?;
        Ok("User registered successfully".to_string())
    }
}

#[tauri::command]
async fn is_authenticated(state: State<'_, Mutex<S2SecretData>>) -> Result<bool, ()> {
    let state = state.lock().await;
    Ok(state.session_id.is_some())
}

async fn create_client_data(state: &mut S2SecretData,app_handle: &tauri::AppHandle, user_id: &Uuid) -> Result<String, ()> {
   let file_path = app_handle.dialog().file().set_file_name("s2secret.sqlite").add_filter("SQLite", &["sqlite"]).blocking_save_file().unwrap();
   let file_path = file_path.as_path().unwrap();
   state.client_local_data_path = file_path.to_str().unwrap().to_string();
   let client_db_connection_options = SqliteConnectOptions::new()
        .filename(file_path)
        .create_if_missing(true);
    let client_db_pool = SqlitePool::connect_with(client_db_connection_options).await.map_err(|_| ())?;
    let mut transaction = client_db_pool.begin().await.map_err(|_| ())?;
    sqlx::query("CREATE TABLE IF NOT EXISTS user (id TEXT PRIMARY KEY, client_key_file BLOB NOT NULL);")
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    sqlx::query("CREATE TABLE IF NOT EXISTS secret (id TEXT PRIMARY KEY, client_share BLOB NOT NULL, client_share_padding BLOB NOT NULL, data_encryption_key BLOB NOT NULL, updated_at INTEGER NOT NULL, user_id TEXT NOT NULL, FOREIGN KEY(user_id) REFERENCES user(id));")
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    sqlx::query("CREATE TABLE IF NOT EXISTS emergency_contact (id TEXT PRIMARY KEY, client_share BLOB NOT NULL, data_encryption_key BLOB NOT NULL, password_salt BLOB NOT NULL, user_id TEXT NOT NULL, FOREIGN KEY(user_id) REFERENCES user(id));")
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    sqlx::query("CREATE TABLE IF NOT EXISTS emergency_contact_access (id_emergency_contact TEXT NOT NULL, id_secret TEXT NOT NULL, data_encryption_key BLOB NOT NULL, ticket_share BLOB NOT NULL, v_share BLOB NOT NULL, a_share BLOB NOT NULL, a BLOB NOT NULL, PRIMARY KEY(id_emergency_contact, id_secret), FOREIGN KEY(id_emergency_contact) REFERENCES emergency_contact(id), FOREIGN KEY(id_secret) REFERENCES secret(id));")
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    let mut client_key_file = [0u8; 32];
    OsRng.fill_bytes(&mut client_key_file);
    sqlx::query("INSERT INTO user (id, client_key_file) VALUES (?, ?) ON CONFLICT(id) DO NOTHING;")
        .bind(user_id.to_string())
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
async fn user_name(state: State<'_, Mutex<S2SecretData>>) -> Result<String, ()> {
    let state = state.lock().await;
    Ok(state.user_name.clone().unwrap_or_default())
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
    state.passwords.clear();
    state.http_client.post("http://localhost:3000/auth/user/logout")
    .send()
    .await
    .map_err(|_| ())?;
    state.session_id = None;
    state.http_client = reqwest_middleware::ClientWithMiddleware::default();
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
async fn select_database_file(state: State<'_, Mutex<S2SecretData>>, app_handle: tauri::AppHandle) -> Result<String, ()> {
    let mut state = state.lock().await;
    let file_path = app_handle.dialog().file().add_filter("SQLite", &["sqlite"]).blocking_pick_file();
    
    if let Some(path) = file_path {
        state.client_local_data_path = path.as_path().unwrap().to_str().unwrap().to_string();
        Ok(state.client_local_data_path.clone())
    } else {
        Err(())
    }
}

#[tauri::command]
async fn login(state: State<'_, Mutex<S2SecretData>>, email: String, master_password: String) -> Result<String, ()> {
    let mut client_rng = OsRng;
    let client_login_start_result = ClientLogin::<DefaultCipherSuite>::start(&mut client_rng, master_password.as_bytes()).unwrap();
    let login_request_bytes = client_login_start_result.message;
    let http_client = reqwest::Client::new();
    let mut buffer = Vec::new();
    let login_initial_request: LoginInitialRequest = LoginInitialRequest {
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
    //let session_id = client_final_response.headers().get("session-id").unwrap().clone();
    let mut state = state.lock().await;
    //state.session_id = Some(uuid::Uuid::parse_str(session_id.to_str().unwrap()).unwrap());
    state.session_key = SecretBox::new(Box::new(Some(client_login_finish_result.session_key.to_vec())));
    state.password_encryption_key = SecretBox::new(Box::new(Some(client_login_finish_result.export_key.to_vec())));
    Ok(temp_session_id.to_str().map_err(|_| ())?.to_string())
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

async fn recover_password(
    state: &S2SecretData,
    secret_id: &Uuid,
) -> Result<String, ()> {
    let mut transaction = SqlitePool::connect(&state.client_local_data_path).await.map_err(|_| ())?.begin().await.map_err(|_| ())?;
    let row = sqlx::query("SELECT client_share, client_share_padding, data_encryption_key FROM secret WHERE id = ?")
        .bind(secret_id.to_string())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    let data_encryption_key: Vec<u8> = decrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), row.get(2)).map_err(|_| ())?;
    let client_share = decrypt(&data_encryption_key, row.get(0)).map_err(|_| ())?;  
    let client_share = Share::try_from(client_share.as_slice()).map_err(|_| ())?;
    let padding_characters_count = decrypt(&data_encryption_key, row.get(1)).map_err(|_| ())?;
    let buffer = Vec::new();
    let server_share_response = state.http_client.get(format!("http://localhost:3000/secrets/{}/share", secret_id))
        .body(buffer)
        .send()
        .await
        .map_err(|_| ())?;
    if server_share_response.status() != 200 {
        return Err(());
    } else {
        let server_share_response = server_share_response.bytes().await.map_err(|_| ())?;
        let server_share_response: SecretShare = ciborium::de::from_reader(server_share_response.as_ref()).map_err(|_| ())?;
        let server_share = Share::try_from(server_share_response.server_share.as_slice()).map_err(|_| ())?;
        let mut shares = vec![client_share, server_share];
        let sharks = Sharks(2);
        let mut secret = sharks.recover(shares.as_slice()).unwrap_or_default();
        secret.truncate(128 - usize::from_le_bytes(padding_characters_count.try_into().map_err(|_| ())?));
        shares.zeroize();
        Ok(String::from_utf8(secret).map_err(|_| ())?)
    }
}

#[tauri::command]
async fn filter_by_search_term(state: State<'_, Mutex<S2SecretData>>, term: String) -> Result<Vec<Password>, ()> {
    let state = state.lock().await;
    let filtered_passwords: Vec<Password> = state.passwords.values()
        .cloned()
        .filter(|password| (password.title.contains(&term) ||
                                           password.user_name.as_ref().is_some_and(|u| u.contains(&term)) ||
                                           password.site.as_ref().is_some_and(|s| s.contains(&term)) ||
                                           password.notes.as_ref().is_some_and(|n| n.contains(&term))))
        .collect();
    Ok(filtered_passwords)
}

#[tauri::command]
async fn reveal_password(
    state: State<'_, Mutex<S2SecretData>>,
    secret_id: Uuid,
) -> Result<String, ()> {
    let state = state.lock().await;
    recover_password(&state, &secret_id).await
}

async fn share_renewal_for_secret(
    state: &S2SecretData,
    secret_id: &Uuid,
) -> Result<String, ()> {
    let mut transaction = SqlitePool::connect(&state.client_local_data_path).await.map_err(|_| ())?.begin().await.map_err(|_| ())?;
    let row = sqlx::query("SELECT client_share, data_encryption_key, updated_at FROM secret WHERE id = ?")
        .bind(secret_id.to_string())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    let data_encryption_key: Vec<u8> = decrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), row.get(1)).map_err(|_| ())?;
    let client_share = decrypt(&data_encryption_key, row.get(0)).map_err(|_| ())?;  
    let mut client_share = Share::try_from(client_share.as_slice()).map_err(|_| ())?;
    let sharks = Sharks(2);
    let client_renewal_shares: Vec<Share> = sharks.proactive_dealer(&client_share).take(2).collect();
    let mut buffer = Vec::new();
    let client_share_renewal_request = ShareRenewal {
        share: Vec::from(&client_renewal_shares[1]),
        updated_at: NaiveDateTime::from_timestamp(row.get(2), 0),
    };
    ciborium::ser::into_writer(&client_share_renewal_request,&mut buffer).map_err(|_| ())?;
    let server_share_renewal_response = state.http_client.post(format!("http://localhost:3000/secrets/{}/renew-share", &secret_id))
        .body(buffer)
        .send()
        .await
        .map_err(|_| ())?;
    if server_share_renewal_response.status() != 200 {
        return Err(());
    } else {
        let response_bytes = server_share_renewal_response.bytes().await.map_err(|_| ())?;
        let server_share_renewal_response: ShareRenewal = ciborium::de::from_reader(response_bytes.as_ref()).map_err(|_| ())?;
        let renewal_share_from_server = Share::try_from(server_share_renewal_response.share.as_slice()).map_err(|_| ())?;
        client_share.renew([&renewal_share_from_server,&client_renewal_shares[0]]).ok();
        update_local_share_on_renewal(&state, &secret_id, &client_share, server_share_renewal_response.updated_at).await?;
        Ok("Secret share renewed successfully".to_string())
    }
}

#[tauri::command]
async fn send_2fa_secret_code(state: State<'_, Mutex<S2SecretData>>, email: String, one_time_secret_code: String, temporal_session_id: String) -> Result<(), ()> {
    let mut state = state.lock().await;
    let mut buffer = Vec::new();
    let one_time_secret_code_request = OneTimeSecretCodeRequest { email, secret_code: one_time_secret_code };
    ciborium::ser::into_writer(&one_time_secret_code_request, &mut buffer).map_err(|_| ())?;
    let two_factor_response = state.http_client.post("http://localhost:3000/auth/user/2fa")
        .body(buffer)
        .header("session-id", &temporal_session_id)
        .send()
        .await
        .map_err(|_| ())?;
    if two_factor_response.status() != 200 {
        return Err(());
    }
    let session_id = two_factor_response.headers().get("session-id").unwrap().clone();
    state.session_id = Some(uuid::Uuid::parse_str(session_id.to_str().unwrap()).unwrap());
    let http_client = Client::builder().build().unwrap();
    state.http_client = ClientBuilder::new(http_client)
    .with(SecureSessionMiddleware {
        session_id: uuid::Uuid::parse_str(session_id.to_str().unwrap()).unwrap(),
        session_key: SecretBox::new(Box::new(state.session_key.expose_secret().as_ref().unwrap().clone()))
    })
    .build();
    Ok(())
}

#[tauri::command]
async fn renew_share(
    state: State<'_, Mutex<S2SecretData>>,
    secret_id: Uuid,
) -> Result<String, ()> {
    let state = state.lock().await;
    share_renewal_for_secret(&state, &secret_id).await
}

async fn add_access_to_emergency_contact_for_secret(state: State<'_, Mutex<S2SecretData>>,id_emergency_contact: Uuid ,secret_id: Uuid,) -> Result<(), ()> {
    let mut v = [0u8; 64];
    OsRng.fill_bytes(&mut v);
    let mut a = [0u8; 64];
    OsRng.fill_bytes(&mut a);
    let password = b"hunter42"; // Bad password; don't actually use!
    let mysecret = b"my secret";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt).ok().unwrap().to_string();
    let mut encryption_key_for_secret = [0u8; 32];
    Argon2::default().hash_password_into(password, &v, &mut encryption_key_for_secret).ok().unwrap();
    let encrypted_secret = encrypt(&encryption_key_for_secret, mysecret).map_err(|_| ())?;
    let ticket = Ticket {
        password_hash: password_hash,
        encrypted_secret,
    };
    let sharks = Sharks(2);
    let config = config::standard();
    let encoded_ticket: Vec<u8> = bincode::encode_to_vec(&ticket, config).unwrap();
    let ticket_shares: Vec<Share> = sharks.dealer(encoded_ticket.as_slice()).take(2).collect();
    let a_shares = sharks.dealer(a.as_slice()).take(2).collect::<Vec<Share>>();
    let v_shares = sharks.dealer(v.as_slice()).take(2).collect::<Vec<Share>>();
    let mut buffer = Vec::new();
    let emergency_access_request = EmergencyAccessRequest {
        id_emergency_contact: id_emergency_contact,
        server_ticket: Vec::from(&ticket_shares[1]),
        server_v: Vec::from(&v_shares[1]),
        server_a: Vec::from(&a_shares[1]),
    };
    ciborium::ser::into_writer(&emergency_access_request,&mut buffer).map_err(|_| ())?;
    let state = state.lock().await;
    let login_initial_response = state.http_client.post(format!("http://localhost:3000/secrets/{}/emergency-contacts", secret_id))
        .body(buffer)
        .send()
        .await
        .map_err(|_| ())?;
   Ok(())
}

#[tauri::command]
async fn copy_password(
    app_handle: tauri::AppHandle,
    state: State<'_, Mutex<S2SecretData>>,
    secret_id: Uuid,
) -> Result<(), ()> {
    let state = state.lock().await;
    let password = recover_password(&state, &secret_id).await?;
    app_handle.clipboard().write_text(password).unwrap();
    Ok(())
}

async fn store_local_emergency_contact_share(
    state: &S2SecretData,
    emergency_contact_id: &uuid::Uuid,
    share: &Share,
) -> Result<(),()> {
    let data_encryption_key = Aes256Gcm::generate_key(OsRng);
    let password_salt = SaltString::generate(&mut OsRng);
    let encrypted_share = encrypt(data_encryption_key.as_ref(), &Vec::from(share)).map_err(|_| ())?;
    let encrypted_password_salt = encrypt(data_encryption_key.as_ref(), password_salt.as_str().as_bytes().as_ref()).map_err(|_| ())?;
    let encrypted_data_encryption_key = encrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), data_encryption_key.as_ref()).map_err(|_| ())?;
    let mut transaction = SqlitePool::connect(&state.client_local_data_path).await.map_err(|_| ())?.begin().await.map_err(|_| ())?;
    sqlx::query("INSERT INTO emergency_contact (id, client_share, data_encryption_key, password_salt, user_id) VALUES (?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET client_share = excluded.client_share, data_encryption_key = excluded.data_encryption_key, password_salt = excluded.password_salt")
        .bind(emergency_contact_id.to_string())
        .bind(encrypted_share)
        .bind(encrypted_data_encryption_key)
        .bind(encrypted_password_salt)
        .bind(state.user_id.unwrap_or_default().to_string())
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    Ok(())
}

async fn password_salt_of_emergency_contact(state: &S2SecretData, emergency_contact_id: &uuid::Uuid) -> Result<String, ()> {
    let mut transaction = SqlitePool::connect(&state.client_local_data_path).await.map_err(|_| ())?.begin().await.map_err(|_| ())?;
    let emergency_contact_row = sqlx::query("SELECT password_salt, data_encryption_key FROM emergency_contact WHERE id = ?")
        .bind(emergency_contact_id.to_string())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    let data_encryption_key = decrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), emergency_contact_row.get(1)).map_err(|_| ())?;
    let password_salt = decrypt(data_encryption_key.as_ref(), emergency_contact_row.get(0)).map_err(|_| ())?;
    Ok(SaltString::from_b64(String::from_utf8(password_salt).map_err(|_| ())?.as_str()).map_err(|_| ())?.as_str().to_owned())
}

async fn data_encryption_key_for_secret(state: &S2SecretData, secret_id: &uuid::Uuid) -> Result<[u8; 32], ()> {
    let mut transaction = SqlitePool::connect(&state.client_local_data_path).await.map_err(|_| ())?.begin().await.map_err(|_| ())?;
    let data_encryption_key = sqlx::query("SELECT data_encryption_key FROM secret WHERE id = ?")
        .bind(secret_id.to_string())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    let data_encryption_key = decrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), data_encryption_key.get(0)).map_err(|_| ())?;
    Ok(data_encryption_key[..32].try_into().map_err(|_| ())?)
}

async fn store_local_emergency_access_data(
    state: &S2SecretData,
    emergency_contact_id: &uuid::Uuid,
    secret_id: &uuid::Uuid,
    ticket_share: &Share,
    v_share: &Share,
    a_share: &Share,
    a: &Vec<u8>
) -> Result<(),()> {
    let mut encryption_key_for_emergency_access = [0u8; 32];
    let password = b"hunter42";
    let password_salt = password_salt_of_emergency_contact(&state, &emergency_contact_id).await?;
    Argon2::default().hash_password_into(password, &password_salt.as_bytes(), &mut encryption_key_for_emergency_access).ok().unwrap();
    let data_encryption_key = data_encryption_key_for_secret(&state, &secret_id).await?;
    let encrypted_data_encryption_key = encrypt(encryption_key_for_emergency_access.as_ref(), data_encryption_key.as_ref()).map_err(|_| ())?;
    let mut transaction = SqlitePool::connect(&state.client_local_data_path).await.map_err(|_| ())?.begin().await.map_err(|_| ())?;
    sqlx::query("INSERT INTO emergency_contact_access (id_emergency_contact, id_secret, data_encryption_key, ticket_share, v_share, a_share, a) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id_emergency_contact, id_secret) DO UPDATE SET data_encryption_key = excluded.data_encryption_key, ticket_share = excluded.ticket_share, v_share = excluded.v_share, a_share = excluded.a_share, a = excluded.a")
        .bind(emergency_contact_id.to_string())
        .bind(secret_id.to_string())
        .bind(encrypted_data_encryption_key)
        .bind(Vec::from(ticket_share))
        .bind(Vec::from(v_share))
        .bind(Vec::from(a_share))
        .bind(a)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    Ok(())
}

async fn store_local_share(
    state: &S2SecretData,
    secret_id: &uuid::Uuid,
    share: &Share,
    padding_characters_count: usize,
) -> Result<(), ()> {
    let data_encryption_key = Aes256Gcm::generate_key(OsRng);
    let encrypted_share = encrypt(data_encryption_key.as_ref(), &Vec::from(share)).map_err(|_| ())?;
    let encrypted_padding_characters_count = encrypt(data_encryption_key.as_ref(), &padding_characters_count.to_le_bytes()).map_err(|_| ())?;
    let encrypted_data_encryption_key = encrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), data_encryption_key.as_ref()).map_err(|_| ())?;
    let mut transaction = SqlitePool::connect(&state.client_local_data_path).await.map_err(|_| ())?.begin().await.map_err(|_| ())?;
    sqlx::query("INSERT INTO secret (id, client_share, client_share_padding, data_encryption_key, updated_at, user_id) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET client_share = excluded.client_share, client_share_padding = excluded.client_share_padding, data_encryption_key=excluded.data_encryption_key, updated_at = excluded.updated_at")
        .bind(secret_id.to_string())
        .bind(encrypted_share)
        .bind(encrypted_padding_characters_count)
        .bind(encrypted_data_encryption_key)
        .bind(Utc::now().timestamp())
        .bind(state.user_id.unwrap_or_default().to_string())
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    Ok(())
}

async fn update_local_share_on_renewal(
    state: &S2SecretData,
    secret_id: &uuid::Uuid,
    share: &Share,
    updated_at: NaiveDateTime,
) -> Result<(), ()> {
    let mut transaction = SqlitePool::connect(&state.client_local_data_path).await.map_err(|_| ())?.begin().await.map_err(|_| ())?;
    let data_encryption_key = sqlx::query("SELECT data_encryption_key FROM secret WHERE id = ?")
        .bind(secret_id.to_string())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ())?;
    let data_encryption_key: Vec<u8> = decrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), data_encryption_key.get(0)).map_err(|_| ())?;
    let encrypted_share = encrypt(&data_encryption_key, &Vec::from(share)).map_err(|_| ())?;
    sqlx::query("UPDATE secret SET client_share = ?, updated_at = ? WHERE id = ?")
        .bind(encrypted_share)
        .bind(updated_at.timestamp())
        .bind(secret_id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    Ok(())
}

async fn delete_local_share(
    state: &S2SecretData,
    secret_id: &uuid::Uuid,
) -> Result<(), ()> {
    let mut transaction = SqlitePool::connect(&state.client_local_data_path).await.map_err(|_| ())?.begin().await.map_err(|_| ())?;
    sqlx::query("DELETE FROM secret WHERE id = ?")
        .bind(secret_id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    Ok(())
}

async fn delete_local_emergency_contact(
    state: &S2SecretData,
    emergency_contact_id: &uuid::Uuid,
) -> Result<(), ()> {
    let mut transaction = SqlitePool::connect(&state.client_local_data_path).await.map_err(|_| ())?.begin().await.map_err(|_| ())?;
    sqlx::query("DELETE FROM emergency_contact_access WHERE id_emergency_contact = ?")
        .bind(emergency_contact_id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    sqlx::query("DELETE FROM emergency_contact WHERE id = ?")
        .bind(emergency_contact_id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    Ok(())
}

#[tauri::command]
async fn delete_secret(state: State<'_, Mutex<S2SecretData>>, secret_id: Uuid) -> Result<String, ()> {
    let mut state = state.lock().await;
    delete_local_share(&state, &secret_id).await.map_err(|_| ())?;
    let delete_secret_response = state.http_client.delete(format!("http://localhost:3000/secrets/{}", &secret_id))
        .send()
        .await
        .map_err(|_| ())?;
    if delete_secret_response.status() != 204 {
        return Err(());
    } else {
        state.passwords.remove(&secret_id);
        Ok("Secret deleted successfully".to_string())
    }
}

#[tauri::command]
async fn delete_emergency_contact(state: State<'_, Mutex<S2SecretData>>, emergency_contact_id: Uuid) -> Result<String, ()> {
    let mut state = state.lock().await;
    delete_local_emergency_contact(&state, &emergency_contact_id).await.map_err(|_| ())?;
    let delete_emergency_contact_response = state.http_client.delete(format!("http://localhost:3000/user/emergency-contacts/{}", &emergency_contact_id))
        .send()
        .await
        .map_err(|_| ())?;
    if delete_emergency_contact_response.status() != 204 {
        return Err(());
    } else {
        state.emergency_contacts.remove(&emergency_contact_id);
        Ok("Emergency contact deleted successfully".to_string())
    }
}

#[tauri::command]
async fn passwords(state: State<'_, Mutex<S2SecretData>>) -> Result<Vec<Password>, ()> {
    let state = state.lock().await;
    Ok(state.passwords.values().cloned().collect())
}

#[tauri::command]
async fn emergency_contacts(state: State<'_, Mutex<S2SecretData>>) -> Result<Vec<EmergencyContact>, ()> {
    let state = state.lock().await;
    Ok(state.emergency_contacts.values().cloned().collect())
}

#[tauri::command]
async fn renew_shares(state: State<'_, Mutex<S2SecretData>>) -> Result<String, ()> {
    let mut state = state.lock().await;
    for (secret_id, password) in &state.passwords {
        if let Some(next_share_update) = password.next_share_update {
            if next_share_update <= Utc::now().naive_utc() {
                //if let Err(_) = share_renewal_for_secret(&state, secret_id).await {
                //    return Err(());
                //}
            }
        }
    }
    Ok("Shares renewed successfully".to_string())
}

#[tauri::command]
async fn load_secret_descriptive_data(state: State<'_, Mutex<S2SecretData>>, secret_id: Uuid) -> Result<String, ()> {
    let mut state = state.lock().await;
    let secret_response = state.http_client.get(format!("http://localhost:3000/secrets/{}", secret_id))
        .send()
        .await
        .map_err(|_| ())?;
    if secret_response.status() != 200 {
        return Err(());
    }
    else {
        let secret_response_bytes = secret_response.bytes().await.map_err(|_| ())?;
        let secret: Secret = ciborium::de::from_reader(secret_response_bytes.as_ref()).map_err(|_| ())?;
        let decrypted_password = secret_descriptive_data(&state, &secret)?;
        state.passwords.insert(secret.id_secret, decrypted_password);
        Ok("Secret descriptive data loaded successfully".to_string())
    }
}




fn secret_descriptive_data(state: &S2SecretData, secret: &Secret) -> Result<Password, ()> {
    let decrypted_title = decrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), &secret.title).map_err(|_| ())?;
    let title = String::from_utf8(decrypted_title).map_err(|_| ())?;
    let mut user_name: Option<String> = None;
    let mut site: Option<String> = None;
    let mut notes: Option<String> = None;
    if let Some(user_name_encrypted) = &secret.user_name {
        let user_name_decrypted = decrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), &user_name_encrypted).map_err(|_| ())?;
        user_name = Some(String::from_utf8(user_name_decrypted).map_err(|_| ())?);
    }
    if let Some(site_encrypted) = &secret.site {
        let decrypted_site = decrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), &site_encrypted).map_err(|_| ())?;
        site = Some(String::from_utf8(decrypted_site).map_err(|_| ())?);
    }
    if let Some(notes_encrypted) = &secret.notes {
        let decrypted_notes = decrypt(state.password_encryption_key.expose_secret().as_ref().unwrap(), &notes_encrypted).map_err(|_| ())?;
        notes = Some(String::from_utf8(decrypted_notes).map_err(|_| ())?);
    }
    Ok(Password {
        id: secret.id_secret,
        title,
        user_name,
        site,
        notes,
        share_updated_at: secret.share_updated_at,
        next_share_update: secret.next_share_update,
        proactive_protection: secret.proactive_protection.clone(),
        password: None, // TODO: Remove. Passwords are not loaded here, only descriptive data
    })
}

#[tauri::command]
async fn load_secrets_descriptive_data(state: State<'_, Mutex<S2SecretData>>) -> Result<String, ()> {
    let mut state = state.lock().await;
    let secrets_response = state.http_client.get(format!("http://localhost:3000/secrets"))
        .send()
        .await
        .map_err(|_| ())?;
    if secrets_response.status() != 200 {
        return Err(());
    }
    else {
        let secrets_response_bytes = secrets_response.bytes().await.map_err(|_| ())?;
        let secrets: Vec<Secret> = ciborium::de::from_reader(secrets_response_bytes.as_ref()).map_err(|_| ())?;
        for secret in secrets {
            let decrypted_password = secret_descriptive_data(&state, &secret)?;
            state.passwords.insert(secret.id_secret, decrypted_password);
        }
        Ok("Load secrets descriptive data successfully".to_string())
    }
}

#[tauri::command]
async fn load_emergency_contacts(state: State<'_, Mutex<S2SecretData>>) -> Result<String, ()> {
    let mut state = state.lock().await;
    let emergency_contacts_response = state.http_client.get(format!("http://localhost:3000/user/emergency-contacts"))
        .send()
        .await
        .map_err(|_| ())?;
    if emergency_contacts_response.status() != 200 {
        return Err(());
    } else {
        let emergency_contacts_response_bytes = emergency_contacts_response.bytes().await.map_err(|_| ())?;
        let emergency_contacts: Vec<EmergencyContact> = ciborium::de::from_reader(emergency_contacts_response_bytes.as_ref()).map_err(|_| ())?;
        for contact in emergency_contacts {
            state.emergency_contacts.insert(contact.id_emergency_contact, contact);
        }
        Ok("Load emergency contacts successfully".to_string())
    }
}

#[tauri::command]
async fn add_secret(state: State<'_, Mutex<S2SecretData>>, title: String, user_name: String, password: String, site: String, notes: String) -> Result<String, ()> {
    let mut state = state.lock().await;
    let password_key = state.password_encryption_key.expose_secret().as_ref().unwrap();
    let (mut shares, padding_characters_count) = secret_padded_shares(&password);
    let mut buffer = Vec::new();

    let new_secret_request = build_secret_upsert_request(
        password_key,
        title,
        user_name,
        &shares[1],
        site,
        notes,
    ).await?;
    ciborium::ser::into_writer(&new_secret_request,&mut buffer).map_err(|_| ())?;
    let add_secret_response = state.http_client.post("http://localhost:3000/secrets")
        .body(buffer)
        .send()
        .await
        .map_err(|_| ())?;
    if add_secret_response.status() != 201 {
        return Err(());
    } else {
        let add_secret_response_bytes = add_secret_response.bytes().await.map_err(|_| ())?;
        let added_secret: Secret = ciborium::de::from_reader(add_secret_response_bytes.as_ref()).map_err(|_| ())?;
        if let Err(_) = store_local_share(&state, &added_secret.id_secret ,&shares[0], padding_characters_count).await {
            return Err(());
        }
        let decrypted_password = secret_descriptive_data(&state, &added_secret)?;
        state.passwords.insert(added_secret.id_secret, decrypted_password);
    }
    shares.zeroize();
    Ok("Secret added successfully".to_string())
}


fn secret_padded_shares(password: &str) -> (Vec<Share>,usize) {
    let sharks = Sharks(2);
    let padding_characters_count = 128 - password.len();
    let mut rng = OsRng;
    let mut random_padding = vec![0u8; padding_characters_count];
    rng.fill_bytes(&mut random_padding);
    let dealer = sharks.dealer([password.as_bytes(), random_padding.as_slice()].concat().as_slice());
    random_padding.zeroize();
    (dealer.take(2).collect(), padding_characters_count)
}

async fn build_secret_upsert_request(
    password_key: &Vec<u8>,
    title: String,
    user_name: String,
    server_share: &Share,
    site: String,
    notes: String,
) -> Result<SecretUpsertRequest, ()> {
    let encrypted_title = encrypt(password_key, title.as_bytes())?;
    let encrypted_user_name = encrypt(password_key, user_name.as_bytes())?;
    let encrypted_site =  encrypt(password_key, site.as_bytes())?;
    let encrypted_notes =  encrypt(password_key, notes.as_bytes())?;
    
    Ok(SecretUpsertRequest {
        title: encrypted_title,
        user_name: if user_name.is_empty() { None } else { Some(encrypted_user_name) },
        site: if site.is_empty() { None } else { Some(encrypted_site) },
        notes: if notes.is_empty() { None } else { Some(encrypted_notes) },
        server_share: Vec::from(server_share),
    })
}

#[tauri::command]
async fn disable_proactive_protection(
    state: State<'_, Mutex<S2SecretData>>,
    secret_id: Uuid,
) -> Result<(), ()> {
    let mut state = state.lock().await;
    let protection_response = state.http_client.post(format!("http://localhost:3000/secrets/{}/disable-proactive-protection", secret_id))
        .send()
        .await
        .map_err(|_| ())?;
    if protection_response.status() != 200 {
        return Err(());
    } else {
        let update_secret_response_bytes = protection_response.bytes().await.map_err(|_| ())?;
        let updated_secret: Secret = ciborium::de::from_reader(update_secret_response_bytes.as_ref()).map_err(|_| ())?;
        let decrypted_password = secret_descriptive_data(&state, &updated_secret)?;
        state.passwords.insert(updated_secret.id_secret, decrypted_password);
        Ok(())
    }
}

#[tauri::command]
async fn add_emergency_contact(
    state: State<'_, Mutex<S2SecretData>>,
    email: String,
    password: String,
    description: Option<String>
) -> Result<(), ()> {
    let mut server_key_file = [0u8; 32];
    OsRng.fill_bytes(&mut server_key_file);
    let sharks = Sharks(2);
    let password_shares = sharks.dealer(password.as_bytes()).take(2).collect::<Vec<Share>>();
    let mut state = state.lock().await;
    let mut buffer = Vec::new();
    ciborium::ser::into_writer(&EmergencyContactRequest {
        email,
        description,
        server_key_file: Vec::from(&server_key_file),
        server_share: Vec::from(&password_shares[1])
    }, &mut buffer).map_err(|_| ())?;
    let contact_response = state.http_client.post("http://localhost:3000/user/emergency-contacts")
        .body(buffer)
        .send()
        .await
        .map_err(|_| ())?;
    if contact_response.status() != 201 {
        return Err(());
    } else {
        let add_emergency_contact_response_bytes = contact_response.bytes().await.map_err(|_| ())?;
        let added_emergency_contact: EmergencyContact = ciborium::de::from_reader(add_emergency_contact_response_bytes.as_ref()).map_err(|_| ())?;
        store_local_emergency_contact_share(&state, &added_emergency_contact.id_emergency_contact, &password_shares[0]).await?;
        state.emergency_contacts.insert(added_emergency_contact.id_emergency_contact, added_emergency_contact);
        Ok(())
    }
}

#[tauri::command]
async fn enable_proactive_protection(
    state: State<'_, Mutex<S2SecretData>>,
    secret_id: Uuid,
    proactive_protection_selected: String,
) -> Result<(), ()> {
    let proactive_protection = match proactive_protection_selected.as_str() {
        "Medium" => ProactiveProtection::Medium,
        "High" => ProactiveProtection::High,
        "Extreme" => ProactiveProtection::Extreme,
        _ => return Err(()),
    };
    let mut state = state.lock().await;
    let mut buffer = Vec::new();
    ciborium::ser::into_writer(&proactive_protection,&mut buffer).map_err(|_| ())?;
    let protection_response = state.http_client.post(format!("http://localhost:3000/secrets/{}/enable-proactive-protection", secret_id))
        .body(buffer)
        .send()
        .await
        .map_err(|_| ())?;
    if protection_response.status() != 200 {
        return Err(());
    } else {
        let update_secret_response_bytes = protection_response.bytes().await.map_err(|_| ())?;
        let updated_secret: Secret = ciborium::de::from_reader(update_secret_response_bytes.as_ref()).map_err(|_| ())?;
        let decrypted_password = secret_descriptive_data(&state, &updated_secret)?;
        state.passwords.insert(updated_secret.id_secret, decrypted_password);
        Ok(())
    }
}

#[tauri::command]
async fn update_secret(state: State<'_, Mutex<S2SecretData>>, id: Uuid, title: String, user_name: String, password: String, site: String, notes: String) -> Result<String, ()> {
    let mut state = state.lock().await;
    let password_key = state.password_encryption_key.expose_secret().as_ref().unwrap();
    let (shares, padding_characters_count) = secret_padded_shares(&password);
    let mut buffer = Vec::new();

    let secret_update_request = build_secret_upsert_request(
        password_key,
        title,
        user_name,
        &shares[1],
        site,
        notes,
    ).await?;
    ciborium::ser::into_writer(&secret_update_request,&mut buffer).map_err(|_| ())?;
    let update_secret_response = state.http_client.put(format!("http://localhost:3000/secrets/{}", id))
        .body(buffer)
        .send()
        .await
        .map_err(|_| ())?;
    if update_secret_response.status() != 200 {
        return Err(());
    } else {
        let update_secret_response_bytes = update_secret_response.bytes().await.map_err(|_| ())?;
        let updated_secret: Secret = ciborium::de::from_reader(update_secret_response_bytes.as_ref()).map_err(|_| ())?;
        if let Err(_) = store_local_share(&state, &updated_secret.id_secret ,&shares[0], padding_characters_count).await {
            return Err(());
        }
        let decrypted_password = secret_descriptive_data(&state, &updated_secret)?;
        state.passwords.insert(updated_secret.id_secret, decrypted_password);
    }
    Ok("Secret updated successfully".to_string())
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
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![login,register_user,
            is_authenticated,
            logout, 
            add_secret,
            update_secret,
            delete_secret,
            delete_emergency_contact,
            logged_user_data,
            user_name,
            passwords,
            filter_by_search_term,
            load_secrets_descriptive_data,
            load_secret_descriptive_data,
            load_emergency_contacts,
            emergency_contacts,
            reveal_password,
            send_2fa_secret_code,
            copy_password,
            select_database_file,
            enable_proactive_protection,
            disable_proactive_protection,
            add_emergency_contact,
            renew_shares,
            renew_share
            ])
        .run(tauri::generate_context!())
        .expect("error while running S2Secret application");
}
