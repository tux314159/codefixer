pub mod login;
pub mod oauth;

use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::{
    Client, EmptyExtraTokenFields, EndpointNotSet, EndpointSet, ExtraTokenFields,
    RevocationErrorResponseType, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
};
use serde::{Deserialize, Serialize};
use url::Url;

/* PUBLIC */

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Disabled = -1,
    Pending = 0,
    Trusted = 1,
    Admin = 2,
    Superadmin = 3,
}

impl From<i64> for Role {
    fn from(r: i64) -> Self {
        match r {
            -1 => Role::Disabled,
            0 => Role::Pending,
            1 => Role::Trusted,
            2 => Role::Admin,
            3 => Role::Superadmin,
            _ => Role::Disabled,
        }
    }
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct User {
    pub id: i64,
    pub username: Option<String>,
    pub google_id: String,
    pub email: String,
    pub role: Role,
}

impl User {
    pub fn is_enabled(&self) -> bool {
        self.role > Role::Disabled && self.username.is_some()
    }
}

#[allow(unused)]
#[derive(Clone, Deserialize)]
pub struct OauthRedirParams {
    code: String,
    state: String,
    scope: String,
}

/* PRIVATE */

#[allow(unused)]
#[derive(Clone, Deserialize)]
struct GoogleIdToken {
    aud: String,
    exp: i64,
    iat: i64,
    iss: String,
    sub: String,
    amr: Option<Vec<String>>,
    auth_time: Option<i64>,
    at_hash: Option<String>,
    azp: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    given_name: Option<String>,
    hd: Option<String>,
    locale: Option<String>,
    name: Option<String>,
    nonce: Option<String>, // TODO implement
    picture: Option<Url>,
    profile: Option<Url>,
}

#[derive(Deserialize, Debug, Serialize)]
struct GoogleTokenExtraFields {
    pub id_token: Option<String>,
}

impl ExtraTokenFields for GoogleTokenExtraFields {}

type OauthSimpleClient = Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<GoogleTokenExtraFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

pub type AuthSession = axum_login::AuthSession<login::Backend>;

pub const USERS_CONFIRM_URI: &str = "/api/v1/auth/users/me";

pub mod post {
    use std::sync::Arc;

    use axum::{Extension, Json, body::Body, response::Response};
    use axum_anyhow::{ApiResult, OptionExt};
    use regex::regex;
    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};

    use super as auth;
    use crate::app;

    #[derive(Deserialize, Serialize)]
    pub struct UsersConfirmParams {
        pub username: String,
    }

    pub async fn users_confirm(
        auth: auth::AuthSession,
        st: Extension<Arc<app::State>>,
        Json(payload): Json<UsersConfirmParams>,
    ) -> ApiResult<Response> {
        let user = auth.user().await.context_bad_request("Invalid user")?;

        if user.username.is_some() {
            return Err(axum_anyhow::conflict(
                "Error confirming user",
                "User has already been confirmed",
            ));
        }

        if !regex!("([[:alnum:]]|_)+").is_match(&payload.username) {
            return Err(axum_anyhow::conflict(
                "Error confirming user",
                "Username contains invalid characters",
            ));
        }

        match sqlx::query!(
            "UPDATE users SET username = ? WHERE id = ?",
            &payload.username,
            user.id
        )
        .execute(&st.db_pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(e)) => {
                if e.is_unique_violation() {
                    Err(axum_anyhow::conflict(
                        "Error confirming user",
                        "Username already exists",
                    ))
                } else {
                    Err(axum_anyhow::conflict(
                        "Error confirming user",
                        "Unknown error",
                    ))
                }
            }
            Err(_) => Err(axum_anyhow::conflict(
                "Error confirming user",
                "Unknown error",
            )),
        }?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap())
    }
}
