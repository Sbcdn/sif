use cardano_serialization_lib::{Transaction, utils::hash_transaction};
use crate::error::SifError;

use crate::models::{LMPTX,MempoolPayload};
use minreq;
use serde_json::json;

pub fn restore_utxos(cbor_txs : &Vec::<String>) -> Result<Vec::<LMPTX>, SifError> {
    let mut out = Vec::<LMPTX>::new();

    for cbor_tx in cbor_txs {
    
        let txb = Transaction::from_bytes(hex::decode(cbor_tx)?)?.body();
        let txhash = hex::encode(hash_transaction(&txb).to_bytes());
        let inputs = txb.inputs();
        let mut utxos = Vec::<String>::new();
        for i in 0..inputs.len() {
            let s = inputs.get(i);
            utxos.push(format!("{}#{}",hex::encode(s.transaction_id().to_bytes()),s.index()));
        }
        out.push(LMPTX::new(txhash,utxos))

    }
    Ok(out)
}


pub fn send_utxos(cbor_tx : &Vec::<String>) -> Result<(), SifError> {
    let utxos = restore_utxos(cbor_tx);
    let network = std::env::var("CARDANO_NETWORK").expect("Environment variable CARDANO_NETWORK not available");
    let nw = match &network[..] {
        "MAINNET" => 1,
        "TESTNET" => 0,
        _ => {
            log::error!("Cannot determine network, assume testnet");
            0
        }
    };

    let sif_server = std::fs::read_to_string("maes")
        .expect("Something went wrong reading the file");
    log::debug!("SIF Server: {}",sif_server);
    match utxos {
        Ok(u) => {
            log::info!("Containing Utxos: {:?}", u);
            let payload= MempoolPayload {
                utxos : u,
                network : nw,
            } ;
            let url = sif_server+"/amem";
            let res = minreq::post(url).with_json(&payload)?.send()?;
            log::debug!("Send Mempool, Repsonse: {:?}",res);
        }

        Err(e) => {
            log::error!("An error occured during determining the utxos of the transaction: {:?}",e.to_string());
        }
    }
    Ok(())

}