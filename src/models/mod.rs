use async_trait::async_trait;

mod pool;
pub mod pool_query;
mod token;

pub use pool::Pool;
use sqlx::{postgres::PgRow, query_as, FromRow, Postgres};
pub use token::Token;

#[async_trait]
pub trait Model: for<'a> FromRow<'a, PgRow> + std::marker::Unpin + Send {
    fn id(&self) -> &str;
    fn table_name() -> String;

    async fn find(id: &str, db_pool: &sqlx::Pool<sqlx::Postgres>) -> Result<Self, sqlx::Error> {
        let find_query = format!("SELECT * FROM {} WHERE id=$1", Self::table_name());
        query_as::<Postgres, Self>(&find_query)
            .bind(id)
            .fetch_one(db_pool)
            .await
    }

    async fn create(&self, db_pool: &sqlx::Pool<sqlx::Postgres>) -> Result<&Self, sqlx::Error>;
    async fn update(&self, db_pool: &sqlx::Pool<sqlx::Postgres>) -> Result<&Self, sqlx::Error>;
    async fn save(&self, db_pool: &sqlx::Pool<sqlx::Postgres>) -> Result<&Self, sqlx::Error> {
        match Self::find(self.id(), db_pool).await {
            Ok(_) => self.update(db_pool).await,
            Err(_) => self.create(db_pool).await,
        }
    }
}
