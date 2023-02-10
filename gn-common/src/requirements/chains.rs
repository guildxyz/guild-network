use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum EvmChain {
    Ethereum = 1,
    Bsc = 56,
    Gnosis = 100,
    Polygon = 137,
}
