pub mod api;
pub mod app;

use crate::api::auth;
use anyhow::Result;
use askama::Template;
use axum::Extension;
use axum::response::Html;
use axum::routing::post;
use axum::{Router, routing::get};
use axum_login::{AuthManagerLayerBuilder, login_required};
use dotenv;
use futures::StreamExt;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tokio::signal;
use tokio::task::AbortHandle;
use tower_sessions::cookie::SameSite;
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;

#[allow(unused)]
#[derive(Debug)]
struct Problem {
    id: i64,
    title: String,
    source: String,
    tl: i64,
    ml: i64,
    runtype: i64,
}

#[derive(Template)]
#[template(path = "problems.html")]
struct ProblemsTemplate<'a> {
    problems: &'a Vec<Problem>,
}

async fn shutdown_signal(abort_jobs: Vec<AbortHandle>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    let abort = || {
        for job in &abort_jobs {
            job.abort();
        }
    };

    tokio::select! {
        () = ctrl_c => { abort() },
        () = terminate => { abort() },
    }
}

async fn get_problems(st: Extension<Arc<app::State>>) -> Html<String> {
    let rows = sqlx::query_as!(Problem, "SELECT * FROM problems").fetch(&st.db_pool);
    let rows = rows.map(|x| x.unwrap()).collect().await;
    let problems_template = ProblemsTemplate { problems: &rows };
    Html(problems_template.render().unwrap())
}

//#[axum::debug_handler]
#[tokio::main]
async fn main() -> Result<()> {
    axum_anyhow::set_expose_errors(true);
    let db_url = dotenv::var("DATABASE_URL").unwrap();
    let pool = SqlitePoolOptions::new().connect(db_url.as_str()).await?;

    // Set up session manager.
    let session_store = SqliteStore::new(pool.clone())
        .with_table_name("client_sessions")
        .unwrap();
    session_store.migrate().await?;
    // Should be a temporary table.
    sqlx::query("DELETE FROM oauth_tokens")
        .execute(&pool)
        .await?;
    let clean_expired_sessions_job = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // change this eventually!
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::SignedDuration::hours(24)));

    let backend = auth::login::Backend {
        db_pool: pool.clone(),
    };
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer.clone()).build();

    let appstate = Arc::new(app::State { db_pool: pool });

    let protected_routes = Router::new()
        .route("/problems", get(get_problems))
        .route_layer(login_required!(
            api::auth::login::Backend,
            login_url = api::auth::login::LOGIN_URI
        ));

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        // API routes.
        // Auth.
        .route(api::auth::login::LOGIN_URI, get(api::auth::login::get::login))
        .route(api::auth::login::LOGOUT_URI, post(api::auth::login::post::logout))
        .route(
            api::auth::oauth::AUTHENTICATE_URI,
            get(api::auth::oauth::get::authenticate),
        )
        .route(api::auth::oauth::CALLBACK_URI, get(api::auth::oauth::get::callback))
        .route(api::problems::PROBLEMS_URI, get(api::problems::get::problems))
        .merge(protected_routes)
        .layer((Extension(appstate.clone()), session_layer, auth_layer));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(vec![
            clean_expired_sessions_job.abort_handle(),
        ]))
        .await?;

    println!("Shutdown");
    appstate.db_pool.close().await;

    Ok(())
}
