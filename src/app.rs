use sqlx::SqlitePool;

pub const SITE_ROOT_URI: &str = "http://localhost:3000";

#[derive(Clone, Debug)]
pub struct State {
    pub db_pool: SqlitePool,
}
