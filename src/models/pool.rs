use async_trait::async_trait;
use bigdecimal::BigDecimal;
use sqlx::{query, FromRow, Postgres};

use super::{pool_query::pools_for_token::poolFields as GqlPoolFields, Model, Token};

#[derive(Clone, FromRow, Eq)]
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
    pub token0_balance: String,
    pub token1_balance: String,
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
                fee_tier,
                token0_balance,
                token1_balance
            ) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            self.id,
            self.token0_id,
            self.token1_id,
            self.token0_price,
            self.token1_price,
            self.total_value_locked_token0,
            self.total_value_locked_token1,
            self.liquidity,
            self.fee_tier,
            self.token0_balance,
            self.token1_balance,
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
                fee_tier,
                token0_balance,
                token1_balance
            ) = ($2, $3, $4, $5, $6, $7, $8, $9, $10, $11) WHERE id = $1",
            self.id,
            self.token0_id,
            self.token1_id,
            self.token0_price,
            self.token1_price,
            self.total_value_locked_token0,
            self.total_value_locked_token1,
            self.liquidity,
            self.fee_tier,
            self.token0_balance,
            self.token1_balance,
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

    pub async fn token0_balance(&self, db_pool: &sqlx::Pool<Postgres>) -> Result<BigDecimal, sqlx::Error> {
        if self.token0_balance.is_empty() {
            return Ok(BigDecimal::from(0));
        }

        let token0_balance: BigDecimal = self.token0_balance.parse().unwrap();
        let token0_decimals: u32 = self.token0(db_pool).await?.decimals.parse().unwrap();
        Ok(token0_balance / (10_i64.pow(token0_decimals)))
    }

    pub async fn token1_balance(&self, db_pool: &sqlx::Pool<Postgres>) -> Result<BigDecimal, sqlx::Error> {
        if self.token1_balance.is_empty() {
            return Ok(BigDecimal::from(0));
        }

        let token1_balance: BigDecimal = self.token1_balance.parse().unwrap();
        let token1_decimals: u32 = self.token1(db_pool).await?.decimals.parse().unwrap();
        Ok(token1_balance / (10_i64.pow(token1_decimals)))
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
            token0_balance: "".to_string(),
            token1_balance: "".to_string(),
        }
    }
}
