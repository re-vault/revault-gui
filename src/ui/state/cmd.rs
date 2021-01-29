use std::sync::Arc;

use crate::revaultd::{
    model::{Vault, VaultTransactions},
    RevaultD, RevaultDError,
};

pub async fn get_blockheight(revaultd: Arc<RevaultD>) -> Result<u64, RevaultDError> {
    revaultd.get_info().map(|res| res.blockheight)
}

pub async fn list_vaults(
    revaultd: Arc<RevaultD>,
) -> Result<Vec<(Vault, VaultTransactions)>, RevaultDError> {
    let vaults = revaultd.list_vaults().map(|res| res.vaults)?;
    let outpoints = vaults.iter().map(|vlt| vlt.outpoint()).collect();
    let txs = revaultd.list_transactions(Some(outpoints))?;

    let mut vec = Vec::new();
    for vlt in vaults {
        if let Some(i) = txs
            .transactions
            .iter()
            .position(|tx| tx.outpoint == vlt.outpoint())
        {
            vec.push((vlt, txs.transactions[i].to_owned()));
        }
    }
    Ok(vec)
}
