use serde::Serialize;

use crate::api::types::{BlockHeader, Cursor, MempoolStatus, Peer, State, Tx};
use blockchain::BlockTx;
use zkvm::TxHeader;

#[derive(Serialize)]
pub struct Status {
    pub mempool: MempoolStatus,
    pub state: State,
    pub peers: Vec<Peer>,
}

#[derive(Serialize)]
pub struct MempoolTxs {
    pub cursor: String,
    pub status: MempoolStatus,
    pub txs: Vec<Tx>,
}

#[derive(Serialize)]
pub struct Blocks {
    pub cursor: String,
    pub blocks: Vec<BlockHeader>,
}

#[derive(Serialize)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<BlockTx>,
}

#[derive(Serialize)]
pub struct TxResponse {
    pub status: TxStatus,
    pub tx: Tx,
}

#[derive(Serialize)]
pub struct TxStatus {
    pub confirmed: bool,
    pub block_height: u64,
    pub block_id: [u8; 32],
}

#[derive(Serialize)]
pub struct Submit {}
