use std::{cmp::min, collections::HashMap};

use async_recursion::async_recursion;
use bigdecimal::BigDecimal;
use sqlx::query_as;
use tracing::{error, info};

use crate::{
    db::db_connection,
    models::{Pool, Token},
};

type DBPool = sqlx::Pool<sqlx::Postgres>;

const MAX_DEPTH: usize = 20;
const MIN_ROOT_AMOUNT: u32 = 100000;

struct Cycle {
    root_token: String,
    pools: Vec<Pool>,
    max_price: BigDecimal,
}

impl Cycle {
    pub fn router_path(&self) -> Vec<String> {
        let mut cur_token = self.root_token.clone();
        let mut path: Vec<String> = vec![];

        for pool in &self.pools {
            let next_token = if pool.is_token_0(&cur_token) {
                pool.token1_id.clone()
            } else {
                pool.token0_id.clone()
            };
            path.push(cur_token);
            path.push(pool.fee_tier.clone());
            cur_token = next_token;
        }

        path.push(cur_token);
        path
    }
}

pub async fn process_cycles(root_token: Token) {
    let mut cycles: Vec<Cycle> = vec![];
    let mut memoized_prices: HashMap<(String, String), (BigDecimal, Vec<Pool>)> = HashMap::new();
    let db_pool = db_connection().await;
    let root_pools = pools_for_token(&db_pool, &root_token.id).await;

    for pool in root_pools {
        let (max_price, price_path) = find_cycle(
            &db_pool,
            &mut memoized_prices,
            &root_token.id,
            pool.clone(),
            root_token.id.clone(),
            vec![pool.clone()],
        )
        .await;
        cycles.push(Cycle {
            root_token: root_token.id.clone(),
            pools: price_path,
            max_price,
        });
    }

    print_cycle_results(cycles);
}

#[async_recursion]
async fn find_cycle(
    db_pool: &DBPool,
    memoized_prices: &mut HashMap<(String, String), (BigDecimal, Vec<Pool>)>,
    root_token_id: &str,
    cur_pool: Pool,
    cur_token_id: String,
    cur_path: Vec<Pool>,
) -> (BigDecimal, Vec<Pool>) {
    if cur_path.len() > MAX_DEPTH {
        return (BigDecimal::from(0), vec![]);
    } else if cur_token_id != root_token_id
        && (cur_pool.token0_id == root_token_id || cur_pool.token1_id == root_token_id)
        && cur_path.len() > 1
    {
        return (cur_pool.fee_price_for(&cur_token_id), vec![cur_pool]);
    } else if memoized_prices.contains_key(&(cur_pool.id.clone(), cur_token_id.clone())) {
        return memoized_prices
            .get(&(cur_pool.id, cur_token_id))
            .unwrap()
            .clone();
    }

    let new_token_id = if cur_pool.is_token_0(&cur_token_id) {
        cur_pool.token1_id.clone()
    } else {
        cur_pool.token0_id.clone()
    };
    let new_pools = pools_for_token(db_pool, &new_token_id).await;
    let mut new_prices: Vec<(BigDecimal, Vec<Pool>)> = vec![];

    for new_pool in new_pools {
        if new_pool.id == cur_pool.id || cur_path.contains(&new_pool) {
            continue;
        }

        let mut new_path = cur_path.clone();
        new_path.push(new_pool.clone());

        new_prices.push(
            find_cycle(
                db_pool,
                memoized_prices,
                root_token_id,
                new_pool,
                new_token_id.clone(),
                new_path,
            )
            .await,
        );
    }

    let (future_max_price, mut future_pool_path) = new_prices
        .iter()
        .max_by_key(|v| &v.0)
        .unwrap_or(&(BigDecimal::from(0), vec![]))
        .clone();
    
    let mut price_pool_path = vec![];
    let token_balance_result = if cur_pool.is_token_0(&cur_token_id) {
        cur_pool.token0_balance(db_pool).await
    } else {
        cur_pool.token1_balance(db_pool).await
    };
    let token_balance = match token_balance_result {
        Ok(balance) => balance,
        Err(err) => {
            error!(pool_id = cur_pool.id, error = err.to_string(), "Error fetching token balance");
            BigDecimal::from(0)
        }
    };
    
    let cur_max_price = if &future_max_price * BigDecimal::from(MIN_ROOT_AMOUNT) >= token_balance {
        BigDecimal::from(0)
    } else {
        &future_max_price * cur_pool.fee_price_for(&cur_token_id)
    };

    if cur_max_price > BigDecimal::from(0) {
        price_pool_path.push(cur_pool.clone());
        price_pool_path.append(&mut future_pool_path);
    }

    memoized_prices.insert(
        (cur_pool.id, cur_token_id),
        (cur_max_price.clone(), price_pool_path.clone()),
    );
    (cur_max_price, price_pool_path)
}

async fn pools_for_token(db_pool: &DBPool, token_id: &str) -> Vec<Pool> {
    match query_as!(
        Pool,
        "SELECT * FROM pools WHERE token0_id=$1 OR token1_id=$1",
        token_id
    )
    .fetch_all(db_pool)
    .await
    {
        Ok(pools) => pools,
        Err(err) => {
            error!(
                token_id,
                error = err.to_string(),
                "[Cycler] failed to fetch pools for token"
            );
            vec![]
        }
    }
}

fn print_cycle_results(mut cycles: Vec<Cycle>) {
    cycles.sort_by_key(|k| &k.max_price.clone() * BigDecimal::from(-1));
    let n_cycles = min(cycles.len(), 10);

    for cycle in &mut cycles[..n_cycles] {
        let pool_ids: Vec<String> = cycle.pools.iter().map(|pool| pool.id.clone()).collect();
        info!(
            projected_profit = format!("{:.5}", cycle.max_price),
            length = cycle.pools.len(),
            "path={:?} pool_ids={:?}", cycle.router_path(), pool_ids
        );
    }
}
