use std::{collections::HashMap, ops::DerefMut};

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
use sqlx::{sqlite::{SqliteConnectOptions, UpdateHookResult},SqlitePool,SqliteTransaction};
use chrono::{NaiveDateTime, Utc};
use sqlx::Row;
use tauri_plugin_clipboard_manager::ClipboardExt;



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
struct SecretUpsertRequest {
    title: Vec<u8>,
    user_name: Option<Vec<u8>>,
    site: Option<Vec<u8>>,
    notes: Option<Vec<u8>>,
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
   let mut state = state.lock().await;
   state.client_local_data_path = "/tmp/s2secret.sqlite".to_string();
   let client_db_connection_options = SqliteConnectOptions::new()
        .filename(&state.client_local_data_path)
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
    let mut client_key_file = [0u8; 32];
    OsRng.fill_bytes(&mut client_key_file);
    sqlx::query("INSERT INTO user (id, client_key_file) VALUES (?, ?) ON CONFLICT(id) DO NOTHING;")
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
async fn renew_share(
    state: State<'_, Mutex<S2SecretData>>,
    secret_id: Uuid,
) -> Result<String, ()> {
    let state = state.lock().await;
    share_renewal_for_secret(&state, &secret_id).await
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
async fn passwords(state: State<'_, Mutex<S2SecretData>>) -> Result<Vec<Password>, ()> {
    let state = state.lock().await;
    Ok(state.passwords.values().cloned().collect())
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
async fn add_secret(state: State<'_, Mutex<S2SecretData>>, title: String, user_name: String, password: String, site: String, notes: String) -> Result<String, ()> {
    let state = state.lock().await;
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
        let add_secret_id_response: UpsertSecretResponse = ciborium::de::from_reader(add_secret_response_bytes.as_ref()).map_err(|_| ())?;
        if let Err(_) = store_local_share(&state, &add_secret_id_response.id_secret ,&shares[0], padding_characters_count).await {
            return Err(());
        }
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
    let state = state.lock().await;
    let protection_response = state.http_client.post(format!("http://localhost:3000/secrets/{}/disable-proactive-protection", secret_id))
        .send()
        .await
        .map_err(|_| ())?;
    if protection_response.status() != 204 {
        return Err(());
    } else {
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
    let state = state.lock().await;
    let mut buffer = Vec::new();
    ciborium::ser::into_writer(&proactive_protection,&mut buffer).map_err(|_| ())?;
    let protection_response = state.http_client.post(format!("http://localhost:3000/secrets/{}/enable-proactive-protection", secret_id))
        .body(buffer)
        .send()
        .await
        .map_err(|_| ())?;
    if protection_response.status() != 204 {
        return Err(());
    } else {
        Ok(())
    }
}

#[tauri::command]
async fn update_secret(state: State<'_, Mutex<S2SecretData>>, id: Uuid, title: String, user_name: String, password: String, site: String, notes: String) -> Result<String, ()> {
    let state = state.lock().await;
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
        let update_secret_id_response: UpsertSecretResponse = ciborium::de::from_reader(update_secret_response_bytes.as_ref()).map_err(|_| ())?;
        if let Err(_) = store_local_share(&state, &update_secret_id_response.id_secret ,&shares[0], padding_characters_count).await {
            return Err(());
        }
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
        .invoke_handler(tauri::generate_handler![login,register_user,
            is_authenticated,
            logout, 
            add_secret,
            update_secret,
            delete_secret, 
            logged_user_data,
            user_name,
            passwords,
            filter_by_search_term,
            load_secrets_descriptive_data,
            load_secret_descriptive_data,
            reveal_password,
            copy_password,
            enable_proactive_protection,
            disable_proactive_protection,
            renew_shares,
            renew_share,
            create_client_data])
        .run(tauri::generate_context!())
        .expect("error while running S2Secret application");
}
