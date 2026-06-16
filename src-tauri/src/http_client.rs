use tauri_plugin_http::reqwest;
use reqwest_middleware::{Middleware, Next, Result as MiddlewareResult};
use reqwest::{Body, Request, Response, Client, Certificate,tls};
use tauri::http::{self, Extensions};
use coset::{CborSerializable, CoseEncrypt0, CoseEncrypt0Builder, HeaderBuilder};
use secrecy::{ExposeSecret, SecretBox};
use aes_gcm::Aes256Gcm;
use aes_gcm::aead::AeadCore;
use rand::rngs::OsRng;
use crate::cryptography;

pub(crate) struct SecureSessionMiddleware {
    pub session_id: uuid::Uuid,
    pub session_key: SecretBox<Vec<u8>>,
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
                                                                    .ciphertext(cryptography::encrypt_with_nonce(self.session_key.expose_secret(), request_body_bytes, nonce).unwrap_or_default())
                                                                    .build();
                *request_body = Body::from(encrypted_body.to_vec().unwrap_or_default());
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
                    let decrypted_request_content = cryptography::decrypt_using_nonce(self.session_key.expose_secret(),&cbor_encrypted_payload,&nonce).unwrap_or_default();
                    let decrypted_body = Body::from(decrypted_request_content);
                    final_response = response_builder.body(decrypted_body).unwrap();
                } else {
                    final_response = response_builder.body(Body::default()).unwrap();
                }
                Ok(Response::from(final_response))
            },
            Err(e) => Err(e),
        }
    }
}

pub fn https_client() -> Client {
    let mut https_client_builder = Client::builder()
                                    .use_rustls_tls()
                                    .https_only(true)
                                    .http2_prior_knowledge()
                                    .min_tls_version(tls::Version::TLS_1_3);
    #[cfg(debug_assertions)]
    {
        // Embed the certificate at compile time
        // This path is relative to the `src-tauri/src` file
        let cert_bytes = include_bytes!("../self_signed_certs/rootCA.pem"); 
        
        // Parse and add as root certificate
        let cert = Certificate::from_pem(cert_bytes)
            .expect("Failed to parse dev certificate");
            
        https_client_builder = https_client_builder
            .add_root_certificate(cert);
    }
    https_client_builder.build().unwrap()
}