use consensus::get_shards;
use crypto::{
    aes_hash::{HashState, MerkleTree},
    hash::{do_hash, Hash},
};
use types::WrapperMsg;

use crate::Context;
use crate::{CTRBCMsg, ProtMsg};
use network::{plaintcp::CancelHandler, Acknowledgement};
use tokio::time::{sleep, Duration};

impl Context {
    // Dealer sending message to everybody
    pub async fn start_init(self: &mut Context, msg: Vec<u8>, instance_id: usize) {
        log::info!(
            "Starting CTRBC Init for instance id {}. My byz status: {} ",
            instance_id,
            self.byz
        );
        let shards = get_shards(msg, self.num_faults + 1, 2 * self.num_faults);
        let zero_shards: Vec<Vec<u8>> = shards.iter().map(|shard| vec![0u8; shard.len()]).collect();

        let merkle_tree = construct_merkle_tree(shards.clone(), &self.hash_context);
        let zero_merkle_tree = construct_merkle_tree(zero_shards.clone(), &self.hash_context);

        let sec_key_map = self.sec_key_map.clone();
        // Sleep to simulate network delay
        // sleep(Duration::from_millis(50)).await;
        for (replica, sec_key) in sec_key_map.into_iter() {
            let ctrbc_msg = CTRBCMsg {
                shard: if self.byz {
                    zero_shards[replica].clone()
                    //shards[replica].clone()
                } else {
                    shards[replica].clone()
                },
                mp: if self.byz {
                    zero_merkle_tree.gen_proof(replica)
                } else {
                    merkle_tree.gen_proof(replica)
                },
                origin: self.myid,
            };

            if replica == self.myid {
                self.handle_init(ctrbc_msg, instance_id).await;
            } else {
                let protocol_msg = ProtMsg::Init(ctrbc_msg, instance_id);
                let wrapper_msg =
                    WrapperMsg::new(protocol_msg.clone(), self.myid, &sec_key.as_slice());
                let cancel_handler: CancelHandler<Acknowledgement> =
                    self.net_send.send(replica, wrapper_msg).await;
                self.add_cancel_handler(cancel_handler);
            }
        }
    }

    pub async fn handle_init(self: &mut Context, msg: CTRBCMsg, instance_id: usize) {
        //send echo
        // self.start_echo(msg.content.clone()).await;
        if !msg.verify_mr_proof(&self.hash_context) {
            log::error!(
                "Invalid Merkle Proof sent by node {}, abandoning RBC instance {}",
                msg.origin,
                instance_id
            );
            return;
        }

        // log::debug!(
        //     "Received Init message {:?} from node {}.",
        //     msg.shard,
        //     msg.origin,
        // );
        // let zero_shards: Vec<Vec<u8>> = shards.iter().map(|shard| vec![0u8; shard.len()]).collect();

        let zero_shards: Vec<Vec<u8>> = (0..self.num_nodes)
            .map(|_| vec![0u8; msg.shard.len()])
            .collect();
        let zero_merkle_tree = construct_merkle_tree(zero_shards.clone(), &self.hash_context);

        let ctrbc_msg = CTRBCMsg {
            shard: if self.byz {
                zero_shards[msg.origin].clone()
            } else {
                msg.shard.clone()
            },
            mp: if self.byz {
                zero_merkle_tree.gen_proof(msg.origin)
            } else {
                msg.mp.clone()
            },
            origin: self.myid,
        };

        if self.crash {
            return;
        }

        // Start echo
        self.handle_echo(ctrbc_msg.clone(), instance_id).await;
        let protocol_msg = ProtMsg::Echo(ctrbc_msg, instance_id);

        self.broadcast(protocol_msg).await;

        // Invoke this function after terminating the protocol.
        //self.terminate("1".to_string()).await;
    }
}

pub fn construct_merkle_tree(shards: Vec<Vec<u8>>, hc: &HashState) -> MerkleTree {
    let hashes_rbc: Vec<Hash> = shards.into_iter().map(|x| do_hash(x.as_slice())).collect();

    MerkleTree::new(hashes_rbc, hc)
}
