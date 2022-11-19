use graphql_client::GraphQLQuery;

type BigInt = String;
type BigDecimal = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "uniswap-schema.json",
    query_path = "queries/pools.graphql"
)]
pub struct PoolsForToken;
