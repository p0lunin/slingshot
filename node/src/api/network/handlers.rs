use crate::api::data::{Cursor, HexId, MempoolStatus, State};
use crate::api::network::{requests, responses};
use std::convert::Infallible;
use crate::bc::BlockchainRef;
use crate::api::response::{ResponseResult, error};
use blockchain::{Mempool, BlockchainState, BlockchainProtocol, BlockHeader, Block};
use zkvm::encoding::ExactSizeEncodable;
use crate::api::data;
use zkvm::Hash;

pub(super) async fn status(bc: BlockchainRef) -> ResponseResult<responses::Status> {
    let bc_state = BlockchainState::make_initial(5, vec![]).0;
    let mempool = &Mempool::new(bc_state.clone(), 5);

    let status = mempool_status(mempool);
    let state = &bc_state;
    let tip = state.tip.clone().into();
    let utreexo = [None; 64];

    let state = State {
        tip,
        utreexo
    };

    let peers = vec![];

    Ok(responses::Status {
        mempool: status,
        state,
        peers
    })
}

pub(super) async fn mempool(cursor: Cursor, bc: BlockchainRef) -> ResponseResult<responses::MempoolTxs> {
    let bc_state = BlockchainState::make_initial(5, vec![]).0;
    let mempool = &Mempool::new(bc_state.clone(), 5);
    let txs_owned = Vec::<blockchain::BlockTx>::new();
    let txs = txs_owned.iter();

    let offset = cursor.cursor.parse::<usize>()
        .map_err(|_| error::invalid_cursor())?;
    let elements = Cursor::DEFAULT_ELEMENTS_PER_PAGE as usize;

    let status = mempool_status(mempool);
    let txs = txs.skip(offset).take(elements).map(|tx| Into::<data::Tx>::into(tx.clone())).collect::<Vec<_>>();

    Ok(responses::MempoolTxs {
        cursor: Cursor { cursor: (offset + elements).to_string() },
        status,
        txs
    })
}

pub(super) async fn blocks(cursor: Cursor, bc: BlockchainRef) -> ResponseResult<responses::Blocks> {
    let blocks_headers = Vec::<BlockHeader>::new();

    let offset = cursor.cursor.parse::<usize>()
        .map_err(|_| error::invalid_cursor())?;
    let count = Cursor::DEFAULT_ELEMENTS_PER_PAGE as usize;

    let headers = blocks_headers.iter().skip(offset).take(count).map(|b| b.clone().into()).collect::<Vec<_>>();
    let new_cursor = Cursor {
        cursor: (offset + count).to_string()
    };
    Ok(responses::Blocks {
        cursor: new_cursor,
        blocks: headers
    })
}

pub(super) async fn block(block_id: HexId, bc: BlockchainRef) -> ResponseResult<responses::Block> {
    let header = BlockHeader::make_initial(0, Hash::default());
    let txs = Vec::<blockchain::BlockTx>::new();
    
    Ok(responses::Block {
        header: header.into(),
        txs
    })
}

pub(super) async fn tx(tx_id: HexId, bc: BlockchainRef) -> ResponseResult<responses::TxResponse> {
    unimplemented!()
}

pub(super) async fn submit(raw_tx: requests::RawTx, bc: BlockchainRef) -> ResponseResult<responses::Submit> {
    unimplemented!()
}

fn mempool_status(mempool: &Mempool) -> MempoolStatus {
    let count = mempool.entries().count() as u64;
    let size = mempool.len() as u64;
    let feerate = 0;

    MempoolStatus {
        count,
        size,
        feerate
    }
}
