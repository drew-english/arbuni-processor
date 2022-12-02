use std::collections::HashSet;
use std::{cmp::min, sync::Arc};

use graphql_client::{GraphQLQuery, Response};
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{error, info};

use crate::db::db_connection;
use crate::models::Token;
use crate::models::{
    pool_query::{pools_for_token, PoolsForToken},
    Model, Pool,
};

const UNISWAP_URL: &str = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3";
const N_WORKERS: usize = 10;

pub async fn find_and_update_all_pools(root_token_address: String) {
    let processed_pools: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));
    let processed_tokens: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));
    let tokens_to_explore: Arc<RwLock<Vec<String>>> =
        Arc::new(RwLock::new(vec![root_token_address]));
    let db_pool = db_connection().await;
    clear_pool_data(&db_pool).await;

    while tokens_to_explore.read().await.len() > 0 {
        let token_addrs: Vec<String> = {
            let mut cur_tokens = tokens_to_explore.write().await;
            let n_tokens = min(cur_tokens.len(), N_WORKERS);
            cur_tokens.drain(0..n_tokens).collect()
        };

        let mut handles: Vec<JoinHandle<()>> = vec![];
        for addr in token_addrs {
            let processed_pools = processed_pools.clone();
            let processed_tokens = processed_tokens.clone();
            let tokens_to_explore = tokens_to_explore.clone();
            let db_pool_clone = db_pool.clone();

            handles.push(tokio::spawn(async move {
                let pools = fetch_pools_for_token(&addr, None, None).await;
                for pool in pools {
                    if processed_pools.read().await.contains(&pool.id) {
                        continue;
                    }

                    save_pool_data(&db_pool_clone, &pool).await;
                    processed_pools.write().await.insert(pool.id.clone());

                    let mut next_token = &pool.token0.id;
                    if next_token == &addr {
                        next_token = &pool.token1.id
                    }

                    if !processed_tokens.read().await.contains(next_token) {
                        tokens_to_explore.write().await.push(next_token.to_string());
                    }

                    processed_tokens.write().await.insert(addr.clone());
                    info!(pool_address = pool.id, "[Explorer] Successfully processed");
                }
            }));
        }

        for handle in handles {
            match handle.await {
                Ok(_) => (),
                Err(err) => error!(error = err.to_string(), "Error during pool processing"),
            };
        }
    }
}

pub async fn fetch_pools_for_token(
    token_address: &str,
    n_pools: Option<i64>,
    min_tvl: Option<String>,
) -> Vec<pools_for_token::poolFields> {
    let query_vars = pools_for_token::Variables {
        token_address: token_address.to_string(),
        n_pools: n_pools.unwrap_or(1000),
        min_tvl: min_tvl.unwrap_or_else(|| "1000".to_string()),
    };

    let data = match query(query_vars).await {
        Ok(res) => res.data,
        Err(err) => {
            error!(error = err.to_string(), "[PoolQuery] Error fetching pools");
            None
        }
    };

    match data {
        Some(mut data) => {
            let mut resulting_pools = data.token0_pools;
            resulting_pools.append(&mut data.token1_pools);
            resulting_pools
        }
        None => vec![],
    }
}

async fn query(
    query_vars: pools_for_token::Variables,
) -> Result<Response<pools_for_token::ResponseData>, reqwest::Error> {
    let token_address = query_vars.token_address.clone();
    use std::time::Instant;
    let now = Instant::now();

    let client = reqwest::Client::new();
    let res = client
        .post(UNISWAP_URL)
        .json(&PoolsForToken::build_query(query_vars))
        .send()
        .await?;

    let duration = format!("{:.3?}", now.elapsed());
    info!(duration, token_address, "[PoolQuery]");

    res.json().await
}

async fn save_pool_data(
    db_pool: &sqlx::Pool<sqlx::Postgres>,
    gql_pool: &pools_for_token::poolFields,
) {
    let pool: Pool = gql_pool.into();

    if pool.token0(db_pool).await.is_err() {
        Token {
            id: gql_pool.token0.id.clone(),
            symbol: gql_pool.token0.id.clone(),
        }
        .save(db_pool)
        .await
        .expect("Failed to save token0");
    }

    if pool.token1(db_pool).await.is_err() {
        Token {
            id: gql_pool.token1.id.clone(),
            symbol: gql_pool.token1.id.clone(),
        }
        .save(db_pool)
        .await
        .expect("Failed to save token0");
    }

    pool.save(db_pool).await.expect("Failed to save pool");
}


async fn clear_pool_data(db_pool: &sqlx::Pool<sqlx::Postgres>) {
    sqlx::query!("DELETE FROM pools").execute(db_pool).await.expect("Failed to clear pools");
    sqlx::query!("DELETE FROM tokens").execute(db_pool).await.expect("Failed to clear tokens");
}