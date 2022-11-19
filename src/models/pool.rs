use async_trait::async_trait;
use sqlx::{query, FromRow, Postgres};

use super::{pool_query::pools_for_token::poolFields as GqlPoolFields, Model, Token};

#[derive(FromRow)]
pub struct Pool {
    id: String,
    token0_id: String,
    token1_id: String,
    token0_price: String,
    token1_price: String,
    total_value_locked_token0: String,
    total_value_locked_token1: String,
    liquidity: String,
    fee_tier: String,
}

#[async_trait]
impl Model for Pool {
    fn id(&self) -> &str {
        &self.id
    }

    fn table_name() -> String {
        "pools".to_string()
    }

    async fn create(&self, db_pool: &sqlx::Pool<Postgres>) -> Result<&Self, sqlx::Error> {
        query!(
            "INSERT INTO pools (
                id,
                token0_id,
                token1_id,
                token0_price,
                token1_price,
                total_value_locked_token0,
                total_value_locked_token1,
                liquidity,
                fee_tier
            ) values ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            self.id,
            self.token0_id,
            self.token1_id,
            self.token0_price,
            self.token1_price,
            self.total_value_locked_token0,
            self.total_value_locked_token1,
            self.liquidity,
            self.fee_tier
        )
        .execute(db_pool)
        .await?;
        Ok(self)
    }

    async fn update(&self, db_pool: &sqlx::Pool<Postgres>) -> Result<&Self, sqlx::Error> {
        query!(
            "UPDATE pools SET (
                token0_id,
                token1_id,
                token0_price,
                token1_price,
                total_value_locked_token0,
                total_value_locked_token1,
                liquidity,
                fee_tier
            ) = ($2, $3, $4, $5, $6, $7, $8, $9) WHERE id = $1",
            self.id,
            self.token0_id,
            self.token1_id,
            self.token0_price,
            self.token1_price,
            self.total_value_locked_token0,
            self.total_value_locked_token1,
            self.liquidity,
            self.fee_tier
        )
        .execute(db_pool)
        .await?;
        Ok(self)
    }
}

impl Pool {
    pub async fn token0(&self, db_pool: &sqlx::Pool<Postgres>) -> Result<Token, sqlx::Error> {
        Token::find(&self.token0_id, db_pool).await
    }

    pub async fn token1(&self, db_pool: &sqlx::Pool<Postgres>) -> Result<Token, sqlx::Error> {
        Token::find(&self.token1_id, db_pool).await
    }
}

impl From<&GqlPoolFields> for Pool {
    fn from(gpf: &GqlPoolFields) -> Self {
        Self {
            id: gpf.id.clone(),
            token0_id: gpf.token0.id.clone(),
            token1_id: gpf.token1.id.clone(),
            token0_price: gpf.token0_price.clone(),
            token1_price: gpf.token1_price.clone(),
            total_value_locked_token0: gpf.total_value_locked_token0.clone(),
            total_value_locked_token1: gpf.total_value_locked_token1.clone(),
            liquidity: gpf.liquidity.clone(),
            fee_tier: gpf.fee_tier.clone(),
        }
    }
}
