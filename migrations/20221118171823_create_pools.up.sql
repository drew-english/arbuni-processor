CREATE TABLE pools (
  id varchar(255) PRIMARY KEY,
  token0_id varchar(255) REFERENCES tokens NOT NULL,
  token1_id varchar(255) REFERENCES tokens NOT NULL,
  token0_price varchar(255) NOT NULL,
  token1_price varchar(255) NOT NULL,
  total_value_locked_token0 varchar(255) NOT NULL,
  total_value_locked_token1 varchar(255) NOT NULL,
  liquidity varchar(255) NOT NULL,
  fee_tier varchar(20) NOT NULL
);

CREATE INDEX idx_pools_on_token0_id ON pools(token0_id);
CREATE INDEX idx_pools_on_token1_id ON pools(token1_id);
