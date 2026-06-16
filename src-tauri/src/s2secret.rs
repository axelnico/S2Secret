use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use bincode::{Decode, Encode};
use secrecy::SecretBox;
use std::collections::HashMap;
use opaque_ke::{CredentialFinalization, CredentialRequest, RegistrationRequest, RegistrationUpload};
use crate::{cryptography::DefaultCipherSuite, http_client::https_client};


#[derive(Deserialize, Serialize)]
pub(crate) struct SecretShare {
    pub server_share: Vec<u8>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize)]
pub(crate) struct EmergencyAccessRequest {
    pub id_emergency_contact: Uuid,
    pub server_ticket: Vec<u8>,
    pub server_v: Vec<u8>,
    pub server_a: Vec<u8>
}

#[derive(Serialize,Deserialize)]
pub(crate) struct EmergencyAccessClientDataRequest {
    pub encrypted_data_encryption_key: Vec<u8>,
    pub encrypted_ticket_share: Vec<u8>,
    pub encrypted_v_share: Vec<u8>,
    pub encrypted_a_share: Vec<u8>,
    pub encrypted_a : Vec<u8>,
    pub password_salt: String,
}

#[derive(Serialize,Deserialize)]
pub(crate) struct S2SecretUserUpsertResponse {
    pub id_user: Uuid
}

#[derive(Deserialize, Serialize, Encode, Decode)]
pub(crate) struct Ticket {
    pub password_hash: String,
    pub encrypted_secret: Vec<u8>,
}


#[derive(Clone,Serialize,Deserialize)]
pub(crate) struct Password {
    pub id: Uuid,
    pub title: String,
    pub user_name: Option<String>,
    pub site: Option<String>,
    pub notes: Option<String>,
    pub password: Option<String>, // TODO: Remove this field, it should not be stored in the passwords map
    pub share_updated_at: NaiveDateTime,
    pub next_share_update: Option<NaiveDateTime>,
    pub proactive_protection: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub(crate) enum ProactiveProtection {
    Medium,
    High,
    Extreme
}

#[derive(Default)]
pub(crate) struct S2SecretData {
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub session_id: Option<uuid::Uuid>,
    pub session_key: SecretBox<Option<Vec<u8>>>,
    pub password_encryption_key: SecretBox<Option<Vec<u8>>>,
    pub http_client: reqwest_middleware::ClientWithMiddleware,
    pub client_local_data_path: String,
    pub passwords: HashMap<Uuid, Password>,
    pub emergency_contacts: HashMap<Uuid, EmergencyContact>,
}

impl S2SecretData {
    pub fn new() -> Self {
         S2SecretData {
            user_id: None,
            user_name: None,
            user_email: None,
            session_id: None,
            session_key: SecretBox::default(),
            password_encryption_key: SecretBox::default(),
            http_client: reqwest_middleware::ClientWithMiddleware::new(https_client(), vec![]),
            client_local_data_path: String::default(),
            passwords: HashMap::default(),
            emergency_contacts: HashMap::default(),
        }
    }
}

#[derive(Serialize,Deserialize)]
pub(crate) struct LoginInitialRequest {
    pub client_identifier: uuid::Uuid,
    pub email: String,
    pub message: CredentialRequest<DefaultCipherSuite>,
}

#[derive(Serialize,Deserialize)]
pub(crate) struct LoginFinalRequest {
    pub email: String,
    pub message: CredentialFinalization<DefaultCipherSuite>,
}

#[derive(Serialize,Deserialize)]
pub(crate) struct UserDataResponse {
    pub id_user: Uuid,
    pub email: String,
    pub name: String,
}
#[derive(Serialize,Deserialize)]
pub(crate) struct UpsertSecretResponse {
    pub id_secret: uuid::Uuid,
}

#[derive(Serialize,Deserialize)]
pub(crate) struct EmergencyContactUpsertResponse {
    pub id_emergency_contact: Uuid
}

#[derive(Serialize,Deserialize)]
pub(crate) struct SecretUpsertRequest {
    pub title: Vec<u8>,
    pub user_name: Option<Vec<u8>>,
    pub site: Option<Vec<u8>>,
    pub notes: Option<Vec<u8>>,
    pub server_share: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EmergencyContactSecretAccessResponse {
    pub title: Vec<u8>,
    pub encrypted_secret: Vec<u8>,
    pub user_name: Option<Vec<u8>>,
    pub site: Option<Vec<u8>>,
    pub notes: Option<Vec<u8>>,
    pub server_v: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct OneTimeSecretCodeRequest {
    pub secret_code: String,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EmergencyContactAccessInfo {
    pub id_emergency_contact: Uuid,
    pub id_secret: Uuid,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EmergencyContactRequest {
    pub email: String,
    pub description: Option<String>,
    pub server_share: Vec<u8>,
}

#[derive(Clone,Deserialize, Serialize)]
pub(crate) struct EmergencyContact {
    pub id_emergency_contact: Uuid,
    pub email: String,
    pub description: Option<String>,
    pub server_share: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EmergencyContactFileAccess {
    pub id_emergency_contact: Uuid,
    pub id_secret: Uuid,
    pub password_salt: String,
    pub data_encryption_key: Vec<u8>,
    pub ticket_share: Vec<u8>,
    pub v_share: Vec<u8>,
    pub a_share: Vec<u8>,
    pub a: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EmergencyContactSecretAccessRequest {
    pub password_hash: String,
    pub contact_prover_mac: Vec<u8>,
    pub contact_ticket_share: Vec<u8>,
    pub contact_prover_mac_share: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct ShareRenewal {
    pub share: Vec<u8>,
    pub updated_at: NaiveDateTime
}

#[derive(Deserialize, Serialize)]
pub(crate) struct SecretPatchRequest {
    pub title: Option<Vec<u8>>,
    pub user_name: Option<Vec<u8>>,
    pub site: Option<Vec<u8>>,
    pub notes: Option<Vec<u8>>,
    pub server_share: Option<Vec<u8>>,
}

#[derive(Serialize,Deserialize)]
pub(crate) struct Secret {
    pub id_secret: Uuid,
    pub title: Vec<u8>,
    pub user_name: Option<Vec<u8>>,
    pub site: Option<Vec<u8>>,
    pub notes: Option<Vec<u8>>,
    pub share_updated_at: NaiveDateTime,
    pub next_share_update: Option<NaiveDateTime>,
    pub proactive_protection: Option<String>,
}

#[derive(Serialize,Deserialize)]
pub(crate) struct TransientSecret {
    pub title: String,
    pub user_name: Option<String>,
    pub site: Option<String>,
    pub notes: Option<String>,
    pub password: String,
}

#[derive(Serialize,Deserialize)]
pub(crate) struct EmergencyContactConfirmationData {
    pub v_share: Vec<u8>,
    pub data_encryption_key: Vec<u8>,
    pub temporal_session_id: String,
    pub secret_id: Uuid,
    pub emergency_contact_id: Uuid,
}


#[derive(Serialize,Deserialize)]
pub(crate) struct UserRegistrationRequest {
    pub name: String,
    pub email: String,
    pub message: RegistrationRequest<DefaultCipherSuite>
}

#[derive(Serialize,Deserialize)]
pub(crate) struct UserRegistrationFinishResult {
    pub name: String,
    pub email: String,
    pub message: RegistrationUpload<DefaultCipherSuite>
}

#[derive(Serialize,Deserialize)]
pub(crate) struct S2SecretLoginParameters {
    pub client_pepper: Vec<u8>,
    pub client_identifier: uuid::Uuid,
    pub server_static_public_key: Vec<u8>
}