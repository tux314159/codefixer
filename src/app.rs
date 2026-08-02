use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct State {
    pub db_pool: SqlitePool,
}
