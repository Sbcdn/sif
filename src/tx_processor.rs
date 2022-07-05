use cardano_serialization_lib::{Transaction, utils::hash_transaction};
use crate::error::SifError;
use crate::models::{LMPTX,MempoolPayload,LocalMempoolTransaction};
use crate::{NETWORK,URL,SIF_STD_OUT,get_network_magic};
use pallas::network::miniprotocols::{TESTNET_MAGIC, MAINNET_MAGIC};
use minreq;

pub async fn restore_tx (cbor : &String) -> Result<(String,Transaction),SifError> {
    let tx = Transaction::from_bytes(hex::decode(cbor)?)?;
    let hash = hex::encode(hash_transaction(&tx.body()).to_bytes());
    Ok((hash,tx))
}

pub async fn restore_utxos(cbor_txs : &Vec::<String>) -> Result<Vec::<LMPTX>, SifError> {
    let mut out = Vec::<LMPTX>::new();
    for cbor_tx in cbor_txs {
        let (txhash,txb) = restore_tx(cbor_tx).await?;
        let inputs = txb.body().inputs();
        let mut utxos = Vec::<String>::new();
        for i in 0..inputs.len() {
            let s = inputs.get(i);
            utxos.push(format!("{}#{}",hex::encode(s.transaction_id().to_bytes()),s.index()));
        }
        out.push(LMPTX::new(txhash,utxos))
    }
    Ok(out)
}

pub async fn get_hash_tx(cbor_txs : &Vec::<String>) -> Result<String,SifError> {
    let mut mempool = Vec::<LocalMempoolTransaction>::new();
    for cbor_tx in cbor_txs {
        let (txhash,_) = restore_tx(cbor_tx).await?;
        mempool.push(LocalMempoolTransaction::new(txhash,cbor_tx.clone()));
    }
    let out = serde_json::to_string(&serde_json::json!(mempool))?;
    Ok(out)
}

pub async fn send_utxos(cbor_tx : &Vec::<String>) -> Result<(), SifError> {
    let utxos = restore_utxos(cbor_tx).await;
    if *SIF_STD_OUT {
        let out = get_hash_tx(cbor_tx).await?;
        log::info!("{}",out);
    }

    let network = match get_network_magic(NETWORK.to_string()) {
        TESTNET_MAGIC => 0,
        MAINNET_MAGIC => 1,
        other => other,
    };
    match utxos {
        Ok(u) => {
            log::debug!("Containing Utxos: {:?}", u);
            let payload= MempoolPayload {
                utxos : u,
                network : network,
            };
            let res = minreq::post(URL.to_string()).with_json(&payload)?.send()?;
            log::debug!("Send Mempool, Repsonse: {:?}",res);
        }
        Err(e) => {
            log::error!("An error occured during determining the utxos of the transaction: {:?}",e.to_string());
        }
    }
    Ok(())
}