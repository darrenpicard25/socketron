use byteorder::{NetworkEndian, ReadBytesExt};
use tokio::io;

use crate::LENGTH_PREFIX_SIZE;

pub fn get_message_length(buf: &[u8]) -> Result<u16, io::Error> {
    (&buf[0..LENGTH_PREFIX_SIZE]).read_u16::<NetworkEndian>()
}

pub fn received_full_message(bytes: &[u8]) -> bool {
    if bytes.len() >= 2 {
        let message_size = get_message_length(bytes).expect("Unable to get message length");
        let data_size = bytes.len() - LENGTH_PREFIX_SIZE;

        if message_size == data_size as u16 {
            return true;
        }
    }

    false
}

pub fn received_partial_message(context_buffer: &[u8], bytes: &[u8]) -> bool {
    if bytes.len() < 2 {
        return true;
    }

    if context_buffer.len() == 0 {
        let message_size = get_message_length(bytes).expect("Unable to get message length");
        let data_size = bytes.len() - LENGTH_PREFIX_SIZE;

        if message_size > data_size as u16 {
            return true;
        }
    } else {
        let message_size =
            get_message_length(context_buffer).expect("Unable to get message length");
        let data_size = bytes.len() - LENGTH_PREFIX_SIZE;
        let received_size = context_buffer.len() + data_size;

        if message_size > received_size as u16 {
            return true;
        }
    }

    false
}

pub fn received_multiple_messages(bytes: &[u8]) -> bool {
    if bytes.len() >= 2 {
        let message_size = get_message_length(bytes).expect("Unable to get message length");
        let data_size = bytes.len() - LENGTH_PREFIX_SIZE;

        if message_size < data_size as u16 {
            return true;
        }
    }

    return false;
}

pub fn received_rest_of_message(bytes_remaining: usize, bytes: &[u8]) -> bool {
    if bytes.len() >= bytes_remaining {
        return true;
    }

    return false;
}

pub fn received_new_message(bytes: &[u8]) -> bool {
    if is_probably_new_message(bytes) {
        return true;
    }

    false
}

fn is_probably_new_message(bytes: &[u8]) -> bool {
    if bytes.len() < 6 {
        return false;
    }

    const VALID_MTIS: [&str; 6] = ["0100", "0120", "0200", "0220", "0420", "0800"];
    let max_message_size = 3_418;
    let maybe_message_size = get_message_length(bytes).expect("Unable to get message length"); //TODO
    let maybe_mti = match String::from_utf8(bytes[2..6].to_vec()) {
        Ok(string) => string,
        Err(_) => return false,
    };

    if maybe_message_size > max_message_size {
        return false;
    }

    if maybe_message_size as usize == (bytes.len() - LENGTH_PREFIX_SIZE) {
        return true;
    }

    if VALID_MTIS.contains(&maybe_mti.as_str()) {
        return true;
    }

    if bytes.len() >= 7 {
        let maybe_bitmap_1_byte_1 = match (&bytes[6..7]).read_u8() {
            Ok(maybe_num) => maybe_num,
            Err(_) => return false,
        };

        // The first three bits of the bitmap should always be 111
        if maybe_bitmap_1_byte_1 >> 5 == 7 {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    fn get_buffer_from_file(path: &str) -> Vec<u8> {
        let f = File::open(path).unwrap();
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();

        buffer
    }
    mod get_message_length {

        use super::get_buffer_from_file;
        use crate::message_helpers::get_message_length;

        #[test]
        fn should_get_proper_length_from_authorization_advise() {
            let buffer =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");

            let results = get_message_length(&buffer);

            assert!(results.is_ok());
            assert_eq!(results.unwrap(), 506);
        }

        #[test]
        fn should_get_proper_length_from_authorization_request() {
            let buffer =
                get_buffer_from_file("sample_messages/i2c-authorization-mastercard-request.bin");

            let results = get_message_length(&buffer);

            assert!(results.is_ok());
            assert_eq!(results.unwrap(), 1540);
        }
    }
    mod received_full_message {
        use super::get_buffer_from_file;
        use crate::message_helpers::received_full_message;

        #[test]
        fn should_return_true_when_given_full_message() {
            let buffer =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");

            let results = received_full_message(&buffer);

            assert!(results);
        }

        #[test]
        fn should_return_false_when_given_more_then_one_message() {
            let mut buffer_1 =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");
            let mut buffer_2 =
                get_buffer_from_file("sample_messages/i2c-authorization-mastercard-request.bin");

            buffer_1.append(&mut buffer_2);
            let results = received_full_message(&buffer_1);

            assert!(!results);
        }

        #[test]
        fn should_return_false_when_given_less_then_full_message() {
            let buffer =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");

            let results = received_full_message(&buffer[..buffer.len() - 1]);

            assert!(!results);
        }
    }
    mod received_partial_message {
        use super::get_buffer_from_file;
        use crate::message_helpers::received_partial_message;

        #[test]
        fn should_return_true_if_new_buffer_less_then_2_bytes() {
            let context_buffer = Vec::new();
            let buffer =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");

            let results = received_partial_message(&context_buffer, &buffer[..1]);

            assert!(results);
        }

        #[test]
        fn should_return_true_if_no_context_buffer_and_partial_new_buffer_passed_in() {
            let context_buffer = Vec::new();
            let buffer =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");

            let results = received_partial_message(&context_buffer, &buffer[..buffer.len() - 1]);

            assert!(results);
        }

        #[test]
        fn should_return_true_if_context_buffer_and_partial_new_buffer_still_do_not_make_message_size(
        ) {
            let buffer =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");

            let slice_point = buffer.len() - 50;
            let context_buffer = &buffer[..slice_point];
            let new_buffer = &buffer[slice_point..buffer.len() - 1];

            let results = received_partial_message(context_buffer, new_buffer);

            assert!(results);
        }

        #[test]
        fn should_return_false_if_no_context_buffer_and_new_bytes_equal_message_length() {
            let buffer =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");

            let results = received_partial_message(&Vec::new(), &buffer);

            assert!(!results);
        }

        #[test]
        fn should_return_false_if_no_context_buffer_and_new_bytes_greater_message_length() {
            let mut buffer_1 =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");
            let buffer_2 =
                get_buffer_from_file("sample_messages/i2c-authorization-mastercard-request.bin");

            buffer_1.append(&mut buffer_2[..1].to_vec());

            let results = received_partial_message(&Vec::new(), &buffer_1);

            assert!(!results);
        }

        #[test]
        fn should_return_false_if_context_buffer_and_new_bytes_equal_message_length() {
            let buffer =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");
            let split_point = 100;

            let results = received_partial_message(&buffer[..split_point], &buffer[split_point..]);

            assert!(!results);
        }

        #[test]
        fn should_return_false_if_context_buffer_and_new_bytes_greater_message_length() {
            let mut buffer_1 =
                get_buffer_from_file("sample_messages/i2c-authorization-advice-request.bin");
            let mut buffer_2 =
                get_buffer_from_file("sample_messages/i2c-authorization-mastercard-request.bin");
            let buff_length = buffer_1.len();

            buffer_1.append(&mut buffer_2);

            let results = received_partial_message(
                &buffer_1[..buff_length],
                &buffer_1[buff_length..buff_length + 2],
            );

            assert!(!results);
        }
    }

    mod received_multiple_messages {}
}
