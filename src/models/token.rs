use async_trait::async_trait;
use sqlx::{query, FromRow, Postgres};

use super::Model;

#[derive(FromRow)]
pub struct Token {
    pub id: String,
    pub symbol: String,
}

#[async_trait]
impl Model for Token {
    fn id(&self) -> &str {
        &self.id
    }

    fn table_name() -> String {
        "tokens".to_string()
    }

    async fn create(&self, db_pool: &sqlx::Pool<Postgres>) -> Result<&Self, sqlx::Error> {
        query!(
            "INSERT INTO tokens (id, symbol) values ($1, $2)",
            self.id,
            self.symbol
        )
        .execute(db_pool)
        .await?;
        Ok(self)
    }

    async fn update(&self, db_pool: &sqlx::Pool<Postgres>) -> Result<&Self, sqlx::Error> {
        query!(
            "UPDATE tokens SET symbol=$2 WHERE id=$1",
            self.id,
            self.symbol
        )
        .execute(db_pool)
        .await?;
        Ok(self)
    }
}
