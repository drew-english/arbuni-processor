use std::env;

pub async fn db_connection() -> sqlx::Pool<sqlx::Postgres> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(env::var("DATABASE_URL").unwrap().as_str())
        .await
        .unwrap()
}
