use serde::{Deserialize, Serialize,Serializer,Deserializer};

type PendingTxHash = String;
type UTxO = String;

#[derive(Debug,Clone)]
pub struct LMPTX(PendingTxHash, Vec::<UTxO>);

impl LMPTX {
    pub fn get_txhash(&self) -> &PendingTxHash {
        &self.0
    }
    pub fn get_utxos(&self) -> &Vec::<UTxO> {
        &self.1
    }

    pub fn new(txh : PendingTxHash, u : Vec::<UTxO>) -> Self {
        LMPTX(txh,u)
    }

}

#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct MempoolPayload {
    pub utxos           : Vec::<LMPTX>,
    pub network         : u64,
}

#[derive(Serialize, Deserialize)]
struct UtxoTupleHelper {
    txhash : PendingTxHash,
    utxos  : Vec::<UTxO> ,
}

impl Serialize for LMPTX {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        UtxoTupleHelper { txhash: self.0.clone(), utxos: self.1.clone() }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LMPTX {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
            .map(|UtxoTupleHelper { txhash, utxos }| LMPTX(txhash, utxos))
    }
}