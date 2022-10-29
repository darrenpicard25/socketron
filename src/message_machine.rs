use crate::{
    message_helpers::{
        get_message_length, received_full_message, received_multiple_messages,
        received_new_message, received_partial_message, received_rest_of_message,
    },
    LENGTH_PREFIX_SIZE,
};

use iso_8583_message::IsoMessage;

#[derive(Debug)]
pub enum State {
    Ready,
    Waiting,
    Delivering,
}

#[derive(Debug)]

struct InnerContext {
    buffer: Vec<u8>,
    waiting_for_bytes: usize,
    messages: Vec<IsoMessage>,
}

impl InnerContext {
    fn reset(&mut self) {
        self.buffer.clear();
        self.waiting_for_bytes = 0;
        // Do not clear messages
    }
    fn get_messages_from_buffer(&mut self, bytes: &[u8]) {
        let mut received_buf = bytes;

        while received_buf.len() > 0 {
            if self.buffer.len() > 0 {
                let partial_message_size = get_message_length(&self.buffer)
                    .expect("Unable to get message length")
                    as usize;
                let partial_received_buf_size = received_buf.len();
                let partial_remaining_buf_size =
                    partial_message_size - (partial_received_buf_size + LENGTH_PREFIX_SIZE);

                self.buffer
                    .append(&mut received_buf[..partial_remaining_buf_size].to_vec());

                received_buf = &received_buf[partial_received_buf_size..];

                if partial_message_size == self.buffer.len() - LENGTH_PREFIX_SIZE {
                    self.messages.push(
                        IsoMessage::from_buffer(std::mem::take(
                            &mut self.buffer[LENGTH_PREFIX_SIZE..].to_vec(),
                        ))
                        .unwrap(),
                    );
                    self.reset();
                }
            } else {
                let message_size = get_message_length(received_buf)
                    .expect("Unable to get message length")
                    as usize;
                let message_size_with_length_header = message_size + LENGTH_PREFIX_SIZE;
                let received_buf_size = received_buf.len();

                if message_size_with_length_header == received_buf_size {
                    self.messages.push(
                        IsoMessage::from_buffer(received_buf[LENGTH_PREFIX_SIZE..].to_vec())
                            .unwrap(),
                    );
                    received_buf = &received_buf[0..0];
                } else if message_size_with_length_header < received_buf_size {
                    self.messages.push(
                        IsoMessage::from_buffer(
                            received_buf[LENGTH_PREFIX_SIZE..message_size_with_length_header]
                                .to_vec(),
                        )
                        .unwrap(),
                    );
                    received_buf = &received_buf[message_size_with_length_header..];
                } else if message_size_with_length_header > received_buf_size {
                    self.buffer.append(&mut received_buf.to_vec());
                    received_buf = &received_buf[0..0];
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct StateMachine<State> {
    inner_context: InnerContext,
    inner_state: State,
}

impl StateMachine<State> {
    pub fn new() -> Self {
        let inner_context = InnerContext {
            buffer: Vec::with_capacity(4096),
            messages: Vec::new(),
            waiting_for_bytes: 0,
        };

        Self {
            inner_context,
            inner_state: State::Ready,
        }
    }

    pub fn process(&mut self, bytes: &[u8]) -> Option<Vec<IsoMessage>> {
        match self {
            StateMachine {
                inner_state: State::Ready,
                ..
            } => self.process_ready(bytes),
            StateMachine {
                inner_state: State::Waiting,
                ..
            } => self.process_waiting(bytes),
            _ => panic!("Unknown StateMachine state at this time"),
        };

        if let StateMachine {
            inner_state: State::Delivering,
            ..
        } = self
        {
            // println!("Getting Messages");
            return self.process_delivering();
        }

        None
    }

    fn process_ready(&mut self, bytes: &[u8]) {
        // println!("Processing Ready: {:?}", bytes);
        if received_full_message(bytes) {
            // println!("Processing Full Message: {:?}", bytes);
            self.inner_state = State::Delivering;
            self.inner_context.get_messages_from_buffer(bytes);

            return;
        }

        if received_partial_message(&self.inner_context.buffer, bytes) {
            // println!("Processing Partial Message: {:?}", bytes);

            self.inner_state = State::Waiting;
            self.inner_context.get_messages_from_buffer(bytes);

            return;
        }

        if received_multiple_messages(bytes) {
            // println!("Processing Multiple Message: {:?}", bytes);

            self.inner_state = State::Delivering;
            self.inner_context.get_messages_from_buffer(bytes);

            return;
        }

        panic!("Not handling load properly. TODO");
    }

    fn process_waiting(&mut self, bytes: &[u8]) {
        if received_new_message(bytes) {
            self.inner_state = State::Ready;
            self.inner_context.reset();

            return;
        }

        if received_partial_message(&self.inner_context.buffer, bytes) {
            self.inner_state = State::Waiting;
            self.inner_context.get_messages_from_buffer(bytes);

            return;
        }

        if received_rest_of_message(self.inner_context.waiting_for_bytes, bytes) {
            self.inner_state = State::Delivering;
            self.inner_context.get_messages_from_buffer(bytes);

            return;
        }

        if received_multiple_messages(bytes) {
            self.inner_state = State::Delivering;
            self.inner_context.get_messages_from_buffer(bytes);

            return;
        }

        panic!("Not handling load properly. TODO");
    }

    fn process_delivering(&mut self) -> Option<Vec<IsoMessage>> {
        // println!("Processing Delivering: {:?}", self);
        let iso_messages = std::mem::take(&mut self.inner_context.messages);
        self.inner_context.messages.clear();

        if self.inner_context.buffer.is_empty() {
            self.inner_state = State::Ready;
            return Some(iso_messages);
        } else {
            self.inner_state = State::Waiting;
            return Some(iso_messages);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StateMachine;

    #[test]
    fn it_works() {
        let _state_machine = StateMachine::new();
    }
}
