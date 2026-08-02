use crate::auth;
use axum_login::{AuthUser, AuthnBackend, UserId};
use sqlx::SqlitePool;

impl AuthUser for auth::User {
    type Id = i64;
    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        return &[]; // TODO
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
                SELECT id, username, google_id, email
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
                SELECT id, username, google_id, email
                FROM users
                WHERE id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await
    }
}

pub mod get {
    use axum::extract::Query;
    use axum::response::{IntoResponse, Redirect, Response};
    use axum_anyhow::ApiResult;
    use serde::Deserialize;
    use tower_sessions::Session;

    #[derive(Clone, Debug, Deserialize)]
    pub struct NextQuery {
        next: String,
    }

    pub async fn login(session: Session, Query(params): Query<NextQuery>) -> ApiResult<Response> {
        session.insert("next", params.next).await?;
        Ok(Redirect::to("/api/auth/oauth/authenticate").into_response())
    }
}
