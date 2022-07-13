
use pallas::network::miniprotocols::{handshake, Agent, Transition, txmonitor, txmonitor::*, run_agent_step, run_agent, TESTNET_MAGIC, MAINNET_MAGIC};
use pallas::network::multiplexer::{bearers::Bearer, agents::*, StdChannelBuffer, StdPlexer};

use crate::tx_processor::send_utxos;

mod tx_processor;
mod models;
pub mod error;

use std::env::var;
use lazy_static::lazy_static;


lazy_static! {
    static ref URL: String = var("SIF_SERVER_URL").unwrap_or("http://34.65.108.52:32001/amem".to_string());
    static ref NETWORK: String = var("CARDANO_NETWORK").unwrap_or("MAINNET".to_string());
    static ref SOCKET: String = var("CARDANO_NODE_SOCKET_PATH").expect("Could not find environment variable 'CARDANO_NODE_SOCKET_PATH', please set your node socket path");
    static ref SIF_STD_OUT: bool = var("SIF_STD_OUT").unwrap_or("true".to_string()).parse::<bool>().expect("Could not parse STD_OUT, values need to be 'true' or 'false' ");
}


fn do_handshake(magic : u64 , mut channel: StdChannelBuffer) {
    let versions = handshake::n2c::VersionTable::v10_and_above(magic);
    let _last = run_agent(handshake::Initiator::initial(versions), &mut channel).unwrap();
}

#[tokio::main]
async fn main() {
    let mut builder = env_logger::Builder::from_env(env_logger::Env::default());
    builder.init();
    
    let magic = get_network_magic(NETWORK.to_string());
    let bearer = Bearer::connect_unix(SOCKET.as_str()).unwrap();
    let mut plexer = StdPlexer::new(bearer);
    let channel0 = plexer.use_channel(0).into();
    let mut channel9 = plexer.use_channel(9).into();
    let mut cmem = Vec::<String>::new();
    plexer.muxer.spawn();
    plexer.demuxer.spawn();

    do_handshake(magic ,channel0);
    
    let txm = LocalTxMonitor::initial(State::StIdle);
    let _ = sif_run_agent(txm, &mut channel9, &mut cmem).await;
}

type Slot = u64;

pub async fn sif_run_agent<C>(agent: LocalTxMonitor, buffer: &mut ChannelBuffer<C>, cmem : &mut Vec<String>) -> Transition<LocalTxMonitor>
where
    C: Channel,
{
    let mut agent = agent.apply_start()?;
    let mut current_snapshot : Option<Slot> = None;
    let mut current_size : Option<MempoolSizeAndCapacity>;
    let mut mempool_tx_storage = Vec::<String>::new();

    while !agent.is_done() {

        match (agent.state.clone(), agent.output.clone()) {
            (txmonitor::State::StIdle, None) => {

            },
            (txmonitor::State::StAcquiring, None)=> {

            },
            (txmonitor::State::StAcquired, None) => { 
                agent.request = Some(MsgRequest::MsgGetSizes);
                
            }, 
            (txmonitor::State::StBusy(StBusyKind::GetSizes), Some(MsgResponse::MsgReplyGetSizes(mempoolsizeandcapacity))) => {
                if mempoolsizeandcapacity.number_of_txs > 0 {
                    log::trace!("Transactions in Mempool try to gather them");
                    agent.request = Some(MsgRequest::MsgNextTx);
                    agent.state = State::StAcquired;
                } else {
                    log::trace!("No Transactions in Mempool");
                    agent.request = None;//Some(MsgRequest::MsgNextTx);
                    agent.state = State::StAcquired;
                    log::trace!("State: {:?}, Request: {:?}, Response: {:?}",agent.state(),agent.request,agent.output);
                }
                current_size = Some(mempoolsizeandcapacity);
                log::info!("Current Mempool Size: {:?}", current_size);
            },
            (txmonitor::State::StBusy(StBusyKind::NextTx), Some(MsgResponse::MsgReplyNextTx(stx))) => {
                log::trace!("Check transaction");
                if let Some(tx)  = stx {
                    log::debug!("Transaction: {}",tx);
                    mempool_tx_storage.push(tx.clone());
                   
                    agent.request = Some(MsgRequest::MsgNextTx);
                    agent.state = State::StAcquired;
                    agent.output = None;
                } else {
                    log::info!("Next Transactions returned None");
                    if mempool_tx_storage.len() > 0 {
                        let new_txs = mempool_tx_storage.iter().filter(|n| !cmem.contains(n)).map(|n| n.clone()).collect::<Vec<_>>();
                        *cmem = mempool_tx_storage.clone();
                        match send_utxos(&new_txs).await{
                            Ok(_) => {
                                log::info!("Sent utxos successfull")
                            },
                            Err(e) => {
                                log::error!("Sending utxos failed!: {:?}",e.to_string());
                            }
                        };
                    }
                    mempool_tx_storage = Vec::<String>::new();

                    agent.request = None;
                    agent.output = None;
                    agent.state = State::StAcquired;
                }
            },
            (txmonitor::State::StBusy(StBusyKind::HasTx), Some(MsgResponse::MsgReplyHasTx(_))) => {
                log::trace!("Has transaction");
                
            },
            (txmonitor::State::StDone,None) => {

            },
            _ => {

            }
        }
        agent = run_agent_step(agent, buffer)?;
        if let Some(snap) = agent.snapshot {
            if let Some(current) = current_snapshot {
                if current != snap {
                    log::info!("Current Snapshot: {:?}",current_snapshot);
                }
            }
        } 
        current_snapshot = agent.snapshot;   
    }
    Ok(agent)
}


pub fn get_network_magic(str: String) -> u64 {
    match &str[..] {
        "MAINNET" => MAINNET_MAGIC,
        "TESTNET" => TESTNET_MAGIC,
        other => {
            other.parse::<u64>().expect("Could not parse network magic")
        }
    }
}