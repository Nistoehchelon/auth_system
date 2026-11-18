pub const API_TOKEN_KEY_NAME: &str = "x-api-key";
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct AuthServerAPIToken(pub String);
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct AuthServerUrl(pub String);
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct AuthServerSecretPrefix(pub String);

pub mod internal_api {
    use crate::API_TOKEN_KEY_NAME;

    #[derive(Debug, strum::EnumString, strum::Display, strum::IntoStaticStr)]
    #[strum(serialize_all = "snake_case")]
    pub enum InternalAPIAction {
        AddOrUpdateUser,
        AddTestUserWithPlaintextPassword,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct AddOrUpdateUserRequest {
        pub user_id: String,
        pub username: String,
        pub password_hash: String,
        pub subscription_expires_at_timestamp: u64,
    }
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum AddOrUpdateUserResponse {
        Added,
        Updated,
    }
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct AddTestUserWithPlaintextPasswordRequest {
        pub username: String,
        pub password: String,
    }
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum AddTestUserWithPlaintextPasswordResponse {
        Added,
    }
    #[derive(Debug, Clone)]
    pub enum SendAddOrUpdateUserRequestError {
        SerializationError(String),
        RequestSendingError(String),
        ServerError(String),
        DeserializationError(String),
    }
    pub async fn send_add_or_update_user_request(
        auth_server_url: String,
        auth_server_secret_prefix: String,
        auth_server_token: String,
        req: AddOrUpdateUserRequest,
    ) -> Result<AddOrUpdateUserResponse, SendAddOrUpdateUserRequestError> {
        let full_url = format!(
            "{auth_server_url}/{auth_server_secret_prefix}/{}",
            InternalAPIAction::AddOrUpdateUser
        );
        let serialized = serde_json::to_vec(&req).map_err(|e| {
            SendAddOrUpdateUserRequestError::SerializationError(format!(
                "Failed conversion to bytes {e:#?}"
            ))
        })?;
        let client = reqwest::Client::new();
        let response = client
            .post(&full_url)
            .header(API_TOKEN_KEY_NAME, auth_server_token)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serialized)
            .send()
            .await
            .map_err(|e| {
                SendAddOrUpdateUserRequestError::RequestSendingError(format!(
                    "HTTP request failed: {e:#?}"
                ))
            })?;
        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await.unwrap_or_default();
            return Err(SendAddOrUpdateUserRequestError::ServerError(format!(
                "Server returned status {status}: {err_body}"
            )));
        }
        let parsed_response = response
            .json::<AddOrUpdateUserResponse>()
            .await
            .map_err(|e| {
                SendAddOrUpdateUserRequestError::DeserializationError(format!(
                    "Failed to parse response JSON: {e:#?}"
                ))
            })?;
        Ok(parsed_response)
    }
}

pub mod public_api {
    pub fn get_full_url(auth_server_url: String, api_action: PublicAPIAction) -> String {
        format!("{auth_server_url}/public/{}", api_action)
    }
    #[derive(Debug, strum::EnumString, strum::Display, strum::IntoStaticStr)]
    #[strum(serialize_all = "snake_case")]
    pub enum PublicAPIAction {
        Login,
        ValidateSessionToken,
        CreateJoinToken,
        ValidateJoinToken,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct LoginRequest {
        pub username: String,
        pub password: String,
    }
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum LoginResponse {
        Success(LoginSuccess),
        Error(LoginError),
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct LoginSuccess {
        pub token: String,
        pub username: String,
        pub user_id: String,
    }
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct LoginError {
        pub kind: LoginErrorKind,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum LoginErrorKind {
        InvalidCredentials,
        SubscriptionExpired,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct VerifySessionTokenRequest {
        pub token: String,
    }
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct VerifySessionTokenResponse {
        pub valid: bool,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct CreateJoinTokenRequest {
        pub session_token: String,
        pub server_nonce: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum CreateJoinTokenResponse {
        Success(CreateJoinTokenSuccess),
        Error(CreateJoinTokenError),
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct CreateJoinTokenSuccess {
        pub join_token: String,
    }
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct CreateJoinTokenError {
        pub kind: CreateJoinTokenErrorKind,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum CreateJoinTokenErrorKind {
        InvalidSessionToken,
    }

    // ----------------------

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct ValidateJoinTokenRequest {
        pub join_token: String,
        pub server_nonce: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum ValidateJoinTokenResponse {
        Valid(ValidateJoinTokenSuccess),
        Invalid,
    }
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct ValidateJoinTokenSuccess {
        pub user_id: String,
        pub username: String,
    }
}
