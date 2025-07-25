use crate::{Context, Msg, ProtMsg};

use super::rbc_state;
use crate::Status;

impl Context {
    // A function's input parameter needs to be borrowed as mutable only when
    // we intend to modify the variable in the function. Otherwise, it need not be borrowed as mutable.
    // In this example, the mut can (and must) be removed because we are not modifying the Context inside
    // the function.

    // Dealer sending message to everybody
    pub async fn start_init(self: &mut Context, input_msg: Vec<u8>, instance_id: usize) {
        let rbc_context = self.rbc_context.entry(instance_id).or_default();
        let status = &rbc_context.status;
        // input msg for instance_id:
        // log::info!(
        //     "INIT: Input message for instance_id {}: {:?}",
        //     instance_id,
        //     input_msg
        // );
        assert!(
            *status == Status::WAITING,
            "INIT: Status is not WAITING for instance id: {:?}",
            instance_id
        );
        rbc_context.status = Status::INIT;
        // Draft a message
        let msg = Msg {
            content: input_msg.clone(),
            origin: self.myid,
        };
        self.handle_init(msg.clone(), instance_id).await;
        // Wrap the message in a type
        // Use different types of messages like INIT, ECHO, .... for the Bracha's RBC implementation
        let protocol_msg = ProtMsg::Init(msg, instance_id);
        // Broadcast the message to everyone
        self.broadcast(protocol_msg).await;
    }

    pub async fn handle_init(self: &mut Context, msg: Msg, instance_id: usize) {
        //send echo
        self.start_echo(msg.content.clone(), instance_id).await;

        // log::info!(
        //     "Received Init message {:?} from node {}.",
        //     msg.content,
        //     msg.origin,
        // );
    }
}
