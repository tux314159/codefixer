pub const AUTHENTICATE_URI: &str = "/api/v1/auth/oauth/authenticate";
pub const CALLBACK_URI: &str = "/api/v1/auth/oauth/callback";

pub mod get {
    use std::sync::Arc;

    use anyhow::{Result, anyhow};
    use axum::Extension;
    use axum::response::{IntoResponse, Redirect, Response};
    use axum_anyhow::{ApiResult, ResultExt};
    use axum_extra::extract::Query;
    use jsonwebtoken as jwt;
    use oauth2::{
        AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
        PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
    };
    use serde::Deserialize;
    use tokio::task;
    use tower_sessions::Session;
    use url::Url;

    use crate::app;
    use crate::auth;

    const OAUTH_LOGIN_WAIT_TIMEOUT: u64 = 120;

    #[allow(unused)]
    #[derive(Clone, Deserialize)]
    struct OpenidConfig {
        issuer: Url,
        authorization_endpoint: Url,
        device_authorization_endpoint: Url,
        token_endpoint: Url,
        userinfo_endpoint: Url,
        revocation_endpoint: Url,
        jwks_uri: Url,
        response_types_supported: Vec<String>,
        response_modes_supported: Vec<String>,
        subject_types_supported: Vec<String>,
        id_token_signing_alg_values_supported: Vec<String>,
        scopes_supported: Vec<String>,
        token_endpoint_auth_methods_supported: Vec<String>,
        claims_supported: Vec<String>,
        code_challenge_methods_supported: Vec<String>,
        grant_types_supported: Vec<String>,
        authorization_response_iss_parameter_supported: bool,
    }

    async fn openid_discovery(url: Url) -> Result<OpenidConfig> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(http_client.get(url).send().await?.json().await?)
    }

    fn get_google_client(openid_config: &OpenidConfig) -> ApiResult<auth::OauthSimpleClient> {
        let gcp_client_id = dotenv::var("GOOGLE_CLIENT_ID")?.to_string();
        let gcp_secret = dotenv::var("GOOGLE_SECRET")?.to_string();
        let client = Client::new(ClientId::new(gcp_client_id))
            .set_client_secret(ClientSecret::new(gcp_secret.to_string()))
            .set_auth_uri(AuthUrl::from_url(
                openid_config.authorization_endpoint.clone(),
            ))
            .set_token_uri(TokenUrl::from_url(openid_config.token_endpoint.clone()))
            .set_redirect_uri(RedirectUrl::new("http://localhost:3000".to_string()).unwrap())
            .set_redirect_uri(RedirectUrl::from_url(
                Url::parse(app::SITE_ROOT_URI)?
                    .join(super::CALLBACK_URI)
                    .unwrap(),
            ));

        Ok(client)
    }

    pub async fn authenticate(st: Extension<Arc<app::State>>) -> ApiResult<String> {
        let openid_config = openid_discovery(
            Url::parse("https://accounts.google.com/.well-known/openid-configuration").unwrap(),
        )
        .await?;
        let client = get_google_client(&openid_config)?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        sqlx::query("INSERT INTO oauth_tokens (token, verifier) VALUES (?, ?)")
            .bind(csrf_token.secret())
            .bind(pkce_verifier.secret())
            .execute(&st.db_pool)
            .await?;

        // Delete row after timeout.
        task::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(OAUTH_LOGIN_WAIT_TIMEOUT)).await;
            let res = sqlx::query("DELETE FROM oauth_tokens WHERE token = ?")
                .bind(csrf_token.secret())
                .execute(&st.db_pool)
                .await;
            if let Ok(r) = res {
                println!("Deleted {} expired tokens", r.rows_affected());
            }
        });

        Ok(auth_url.to_string())
    }

    pub async fn callback(
        st: Extension<Arc<app::State>>,
        auth: auth::AuthSession,
        session: Session,
        Query(params): Query<auth::OauthRedirQueryParams>,
    ) -> ApiResult<Response> {
        // Atomic SELECT then DELETE.
        let mut tx = st.db_pool.begin().await?;
        let verifier = sqlx::query_scalar("SELECT verifier FROM oauth_tokens WHERE token = ?")
            .bind(&params.state)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(anyhow!("Invalid CSRF token"))?;
        sqlx::query("DELETE FROM oauth_tokens WHERE token = ?")
            .bind(&params.state)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        let openid_config = openid_discovery(
            Url::parse("https://accounts.google.com/.well-known/openid-configuration").unwrap(),
        )
        .await?;
        let client = get_google_client(&openid_config)?;

        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        let exchange_resp = client
            .exchange_code(AuthorizationCode::new(params.code))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier))
            .request_async(&http_client)
            .await
            .context_internal("token exchange failure")?;
        let id_token = match &exchange_resp.extra_fields().id_token {
            Some(jwt) => jwt::dangerous::insecure_decode::<auth::GoogleIdToken>(jwt)
                .map_err(|_| anyhow!("Failed to decode ID token")),
            None => Err(anyhow!("Failed to get ID token")),
        }?;

        let secret = exchange_resp.access_token().clone().into_secret();
        let redirect_path = session.remove("next").await?.unwrap_or("/".to_string());

        let user = auth
            .authenticate(auth::login::Credentials {
                user_google_id: id_token.claims.sub,
            })
            .await?;
        match user {
            Some(u) => auth.login(&u).await?,
            None => todo!(),
        }
        Ok(Redirect::to(&redirect_path).into_response())
    }
}
