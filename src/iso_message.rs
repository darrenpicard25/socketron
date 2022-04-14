#[derive(Clone)]
pub struct IsoMessage {
    original_message_buffer: Vec<u8>,
}

impl IsoMessage {
    pub fn new(original_message_buffer: Vec<u8>) -> Self {
        Self {
            original_message_buffer,
        }
    }
}
