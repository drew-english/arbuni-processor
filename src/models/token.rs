use async_trait::async_trait;
use sqlx::{query, FromRow, Postgres};

use super::Model;

#[derive(FromRow)]
pub struct Token {
    pub id: String,
    pub symbol: String,
    pub decimals: String,
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
            "INSERT INTO tokens (id, symbol, decimals) values ($1, $2, $3)",
            self.id,
            self.symbol,
            self.decimals
        )
        .execute(db_pool)
        .await?;
        Ok(self)
    }

    async fn update(&self, db_pool: &sqlx::Pool<Postgres>) -> Result<&Self, sqlx::Error> {
        query!(
            "UPDATE tokens SET (symbol, decimals) = ($2, $3)  WHERE id=$1",
            self.id,
            self.symbol,
            self.decimals
        )
        .execute(db_pool)
        .await?;
        Ok(self)
    }
}
