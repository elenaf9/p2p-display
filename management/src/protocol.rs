pub use message::control_message::Alias;
pub use message::control_message::MessageType;
pub use message::control_message::NetworkState;
pub use message::control_message::StoreMessage;
pub use message::ControlMessage;

mod message {
    include!(concat!(env!("OUT_DIR"), "/management.control_message.rs"));
}

impl ControlMessage {
    pub fn new<T: Into<String>>(message_type: MessageType, payload: T) -> Self {
        ControlMessage {
            message_type: message_type as i32,
            payload: payload.into(),
            state: None,
            message: None,
        }
    }
}
