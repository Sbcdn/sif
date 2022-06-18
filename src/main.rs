use pallas_miniprotocols::{handshake, Agent, Transition, txmonitor, txmonitor::*, run_agent_step, run_agent, TESTNET_MAGIC, MAINNET_MAGIC};
use pallas_multiplexer::{self, bearers::Bearer, agents::*};

use crate::utxo_store::send_utxos;

mod utxo_store;
mod models;
pub mod error;



fn do_handshake(magic : u64 , mut channel: pallas_multiplexer::StdChannelBuffer) {
    let versions = handshake::n2c::VersionTable::v10_and_above(magic);
    let _last = run_agent(handshake::Initiator::initial(versions), &mut channel).unwrap();
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    //    /home/tp/Downloads/cardano-node-1.35.0/testnode.socket
    let socket = std::env::var("CARDANO_NODE_SOCKET_PATH").expect("Could not find env CARDANO_NODE_SOCKET_PATH");
    let network = std::env::var("CARDANO_NETWORK").expect("Could not find env CARDANO_NETWORK");

    let magic = match &network[..] {
        "MAINNET" => MAINNET_MAGIC,
        "TESTNET" => TESTNET_MAGIC,
        other => {
            other.parse::<u64>().expect("Could not parse network magic")
        }
    };

    let bearer = Bearer::connect_unix(socket).unwrap();
    let mut plexer = pallas_multiplexer::StdPlexer::new(bearer);
    let channel0 = plexer.use_channel(0).into();
    let mut channel9 = plexer.use_channel(9).into();

    plexer.muxer.spawn();
    plexer.demuxer.spawn();

    do_handshake(magic ,channel0);
    
    let txm = LocalTxMonitor::initial(State::StIdle);
    let _ = sif_run_agent(txm, &mut channel9);
}

type Slot = u64;

pub fn sif_run_agent<C>(agent: LocalTxMonitor, buffer: &mut ChannelBuffer<C>) -> Transition<LocalTxMonitor>
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
                        match send_utxos(&mempool_tx_storage){
                            Ok(_) => {
                                log::info!("Sending utxos successfull!")
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