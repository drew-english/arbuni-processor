use std::{env, time::Duration};

use ethers_core::types::U256;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{error, info};

use crate::{
    db::db_connection,
    models::{Model, Pool},
};

const MAX_BATCH_REQUESTS: usize = 100;
const BALANCE_OF_SELECTOR: &str = "0x70a08231000000000000000000000000"; // sha3('balanceOf(address)').slice(0,10) + 000000000000000000000000
const TOKEN0_REQ_ID_PREFIX: &str = "token0_";
const TOKEN1_REQ_ID_PREFIX: &str = "token1_";

#[derive(Deserialize, Debug)]
struct BalanceRPCReponse {
    id: String,
    result: U256,
}

pub async fn find_and_update_all_balances() {
    let db_pool = db_connection().await;
    let mut req_bodies: Vec<Value> = vec![];
    let mut cur_req_body: Vec<Value> = vec![];
    let pools = fetch_all_pools(&db_pool).await;

    for pool in pools {
        cur_req_body.push(balance_rpc_body(
            &pool.id,
            &pool.token0_id,
            TOKEN0_REQ_ID_PREFIX.to_string() + &pool.id,
        ));
        cur_req_body.push(balance_rpc_body(
            &pool.id,
            &pool.token1_id,
            TOKEN1_REQ_ID_PREFIX.to_string() + &pool.id,
        ));

        if cur_req_body.len() == MAX_BATCH_REQUESTS {
            req_bodies.push(Value::Array(cur_req_body));
            cur_req_body = vec![];
        }
    }
    if !cur_req_body.is_empty() {
        req_bodies.push(Value::Array(cur_req_body));
    }

    let eth_node_url = env::var("PROD_ETH_NODE_URL").unwrap();
    let n_requests = req_bodies.len();
    for (i, body) in req_bodies.iter().enumerate() {
        let req_client = reqwest::Client::new();
        let res = req_client.post(&eth_node_url).json(&body).send().await;

        if let Err(err) = res {
            error!(
                error = err.to_string(),
                "Error on request, stopping further requests"
            );
            break;
        }
        let response = res.unwrap();
        if !response.status().is_success() {
            error!(
                response_body = response.text().await.unwrap(),
                "Bad response status, sleeping then continuing"
            );
            tokio::time::sleep(Duration::from_millis(1200)).await;
            continue;
        }

        let balance_responses = parse_rpc_balance_responses(response).await;
        for balance_res in balance_responses {
            let is_token_0 = &balance_res.id[0..7] == TOKEN0_REQ_ID_PREFIX;
            let pool_id = balance_res.id.split('_').collect::<Vec<&str>>()[1];
            match Pool::find(&db_pool, pool_id).await {
                Ok(mut pool) => {
                    if is_token_0 {
                        pool.token0_balance = balance_res.result.to_string();
                    } else {
                        pool.token1_balance = balance_res.result.to_string();
                    }
                    if let Err(err) = pool.save(&db_pool).await {
                        error!(error = err.to_string(), "Error saving pool balance");
                    }
                }
                Err(err) => error!(error = err.to_string(), "Error finding pool"),
            };
        }

        info!("Finished processing request={}/{}", i + 1, n_requests);
        tokio::time::sleep(Duration::from_millis(1200)).await;
    }
}

async fn fetch_all_pools(db_pool: &sqlx::Pool<sqlx::Postgres>) -> Vec<Pool> {
    sqlx::query_as!(Pool, "SELECT * FROM pools")
        .fetch_all(db_pool)
        .await
        .expect("Failed to fetch all pools")
}

fn balance_rpc_body(owner: &str, token_addr: &str, rpc_id: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": rpc_id,
        "method": "eth_call",
        "params": [
            {
                "data": BALANCE_OF_SELECTOR.to_string() + &owner[2..],
                "to": token_addr,
            },
            "latest"
        ],
    })
}

async fn parse_rpc_balance_responses(response: reqwest::Response) -> Vec<BalanceRPCReponse> {
    let parsed_response = response.json::<Vec<Value>>().await.unwrap();
    let (balance_responses, errors): (Vec<_>, Vec<_>) = parsed_response
        .iter()
        .map(|val| -> Result<BalanceRPCReponse, serde_json::Error> {
            serde_json::value::from_value(val.clone())
        })
        .partition(Result::is_ok);
    let balance_responses: Vec<BalanceRPCReponse> =
        balance_responses.into_iter().map(Result::unwrap).collect();
    let errors: Vec<serde_json::Error> = errors.into_iter().map(Result::unwrap_err).collect();

    for err in errors {
        error!(error = err.to_string(), "Error parsing rpc response");
    }

    balance_responses
}
