pub mod login;
pub mod oauth;

use std::collections::HashSet;

use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::{
    Client, EmptyExtraTokenFields, EndpointNotSet, EndpointSet, ExtraTokenFields,
    RevocationErrorResponseType, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
};
use serde::{Deserialize, Serialize};
use url::Url;

/* PUBLIC */

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    Disabled = -1,
    Pending = 0,
    User = 1,
    Admin = 2,
    Superadmin = 3,
}

impl From<i64> for Role {
    fn from(r: i64) -> Self {
        match r {
            -1 => Role::Disabled,
            0 => Role::Pending,
            1 => Role::User,
            2 => Role::Admin,
            3 => Role::Superadmin,
            _ => Role::Disabled,
        }
    }
}

/// A registered user on the site.
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

impl AuthUser for User {
    type Id = i64;
    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &[] // TODO
    }
}

/// Authentication and authorisation backend.
#[derive(Clone)]
pub struct Backend {
    pub db_pool: sqlx::SqlitePool,
}

/// Credentials used with the backend.
#[derive(Clone)]
pub struct Credentials {
    pub user_google_id: String,
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        Credentials { user_google_id }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        sqlx::query_as!(
            User,
            r#"
                SELECT id, username, google_id, email, role
                FROM users
                WHERE google_id = ?
            "#,
            user_google_id
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        sqlx::query_as!(
            User,
            r#"
                SELECT id, username, google_id, email, role
                FROM users
                WHERE id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await
    }
}

impl AuthzBackend for Backend {
    type Permission = Role;

    async fn get_user_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let role = sqlx::query_scalar!(
            r#"
                SELECT role
                FROM users
                WHERE id = ?
            "#,
            user.id
        )
        .fetch_one(&self.db_pool)
        .await?;
        Ok(HashSet::from([Role::from(role)]))
    }

    async fn has_perm(
        &self,
        user: &Self::User,
        perm: Self::Permission,
    ) -> Result<bool, Self::Error> {
        Ok(self.get_all_permissions(user).await?.contains(&perm))
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

pub type AuthSession = axum_login::AuthSession<Backend>;

pub const USERS_CONFIRM_URI: &str = "/api/v1/auth/users/me";

pub mod post {
    use std::sync::Arc;

    use axum::{Extension, Json, body::Body, response::Response};
    use axum_anyhow::{ApiResult, OptionExt};
    use regex::regex;
    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};

    use crate::app;

    #[derive(Deserialize, Serialize)]
    pub struct UsersConfirmParams {
        pub username: String,
    }

    pub async fn users_confirm(
        auth: super::AuthSession,
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
