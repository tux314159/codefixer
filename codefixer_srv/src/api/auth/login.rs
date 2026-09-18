use axum_login::{AuthUser, AuthnBackend, UserId};
use sqlx::SqlitePool;

use crate::auth;

impl AuthUser for auth::User {
    type Id = i64;
    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &[] // TODO
    }
}

#[derive(Clone)]
pub struct Backend {
    pub db_pool: SqlitePool,
}

#[derive(Clone)]
pub struct Credentials {
    pub user_google_id: String,
}

impl AuthnBackend for Backend {
    type User = auth::User;
    type Credentials = Credentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        Credentials { user_google_id }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        sqlx::query_as!(
            auth::User,
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
            auth::User,
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

pub const LOGIN_URI: &str = "/api/v1/auth/login";
pub const LOGOUT_URI: &str = "/api/v1/auth/logout";

pub mod get {
    use axum::response::{IntoResponse, Redirect, Response};
    use axum_anyhow::ApiResult;
    use axum_extra::extract::Query;
    use serde::Deserialize;
    use tower_sessions::Session;

    #[derive(Clone, Debug, Deserialize, utoipa::IntoParams)]
    pub struct NextParams {
        next: String,
    }

    /// Log a user in; currently only supports OAuth2 with Google.
    #[utoipa::path(
        get,
        path = super::LOGIN_URI,
        params(NextParams),
        responses((status = OK)),
    )]
    pub async fn login(session: Session, Query(params): Query<NextParams>) -> ApiResult<Response> {
        session.insert("next", params.next).await?;
        Ok(Redirect::to("/api/v1/auth/oauth/authenticate").into_response())
    }
}

pub mod post {
    use axum::{body::Body, response::Response};
    use axum_anyhow::ApiResult;
    use reqwest::StatusCode;

    use crate::auth;

    /// Log a user out. Does nothing if the user is not logged in.
    #[utoipa::path(
        post,
        path = super::LOGOUT_URI,
        responses((status = OK))
    )]
    pub async fn logout(auth: auth::AuthSession) -> ApiResult<Response> {
        auth.logout().await?;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap())
    }
}
