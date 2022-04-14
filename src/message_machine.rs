use crate::{
    iso_message::IsoMessage,
    message_helpers::{
        get_message_length, received_full_message, received_multiple_messages,
        received_new_message, received_partial_message, received_rest_of_message,
    },
    LENGTH_PREFIX_SIZE,
};

pub enum State {
    Ready,
    Waiting,
    Delivering,
}

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
        let mut messages: Vec<IsoMessage> = Vec::new();

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
                    messages.push(IsoMessage::new(self.buffer.clone()));
                    self.reset();
                }
            } else {
                let message_size = get_message_length(received_buf)
                    .expect("Unable to get message length")
                    as usize;
                let message_size_with_length_header = message_size + LENGTH_PREFIX_SIZE;
                let received_buf_size = received_buf.len();

                if message_size_with_length_header == received_buf_size {
                    messages.push(IsoMessage::new(received_buf.to_vec()));
                    received_buf = &received_buf[0..0];
                } else if message_size_with_length_header < received_buf_size {
                    messages.push(IsoMessage::new(
                        received_buf[0..message_size_with_length_header].to_vec(),
                    ));
                    received_buf = &received_buf[message_size_with_length_header..];
                } else if message_size_with_length_header > received_buf_size {
                    self.buffer.append(&mut received_buf.to_vec());
                    received_buf = &received_buf[0..0];
                }
            }
        }
    }
}

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
            return self.process_delivering();
        }

        None
    }

    fn process_ready(&mut self, bytes: &[u8]) {
        if received_full_message(bytes) {
            self.inner_state = State::Delivering;
            self.inner_context.get_messages_from_buffer(bytes);

            return;
        }

        if received_partial_message(&self.inner_context.buffer, bytes) {
            self.inner_state = State::Waiting;
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
        let iso_messages = self.inner_context.messages.clone();
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
