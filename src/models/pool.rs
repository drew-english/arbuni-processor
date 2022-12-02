use async_trait::async_trait;
use bigdecimal::BigDecimal;
use sqlx::{query, FromRow, Postgres};

use super::{pool_query::pools_for_token::poolFields as GqlPoolFields, Model, Token};

#[derive(Clone, FromRow, Eq, PartialOrd, Ord)]
pub struct Pool {
    pub id: String,
    pub token0_id: String,
    pub token1_id: String,
    pub token0_price: String,
    pub token1_price: String,
    pub total_value_locked_token0: String,
    pub total_value_locked_token1: String,
    pub liquidity: String,
    pub fee_tier: String,
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
        Token::find(db_pool, &self.token0_id).await
    }

    pub async fn token1(&self, db_pool: &sqlx::Pool<Postgres>) -> Result<Token, sqlx::Error> {
        Token::find(db_pool, &self.token1_id).await
    }

    // pub fn swap(&self, amount: &BigDecimal, zero_for_one: bool) -> BigDecimal {
    //     let fee_bp: BigDecimal = self.fee_tier.parse().unwrap();
    //     let mil: BigDecimal = 1_000_000.into();
    //     let price: BigDecimal = if zero_for_one {
    //         self.token1_price.parse().unwrap()
    //     } else {
    //         self.token0_price.parse().unwrap()
    //     };

    //     (amount * price) * (BigDecimal::from(1) - fee_bp / &mil)
    // }

    pub fn is_token_0(&self, token_id: &str) -> bool {
        token_id == self.token0_id
    }

    pub fn fee_price_for(&self, token_id: &str) -> BigDecimal {
        let mil: BigDecimal = 1_000_000.into();
        let fee_bp: BigDecimal = self.fee_tier.parse().unwrap();
        let fee_percentage = BigDecimal::from(1) - fee_bp / &mil;
        let base_price: BigDecimal = if token_id == self.token0_id {
            self.token1_price.parse().unwrap()
        } else {
            self.token0_price.parse().unwrap()
        };

        base_price * fee_percentage
    }
}

impl PartialEq for Pool {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
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
