use byteorder::{NetworkEndian, ReadBytesExt};
use socketron::IsoMessage;
use std::net::SocketAddr;

use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

const SOCKET_PORT: u16 = 8006;
const LENGTH_PREFIX_SIZE: usize = 2;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], SOCKET_PORT));
    let listener = TcpListener::bind(addr).await?;

    println!("TcpServer started up on {}", addr);

    while let Ok((stream, connection_addr)) = listener.accept().await {
        println!("Connection made on {}", connection_addr);

        match handle_connection(stream).await {
            Ok(_) => {
                println!("Successfully handled connection on {}", connection_addr)
            }
            Err(e) => {
                println!(
                    "An {} error occurred handling connection on {}. Dropping connection",
                    e, connection_addr
                );
            }
        }
    }
    Ok(())
}

async fn handle_connection(mut stream: TcpStream) -> Result<(), io::Error> {
    let (reader, _writer) = stream.split();

    reader.readable().await?;
    let mut context_buf: Vec<u8> = Vec::with_capacity(3_418_usize + 2); // Based on ISO 8583 spec
    let mut temp_buf = [0; 4096];

    loop {
        let message = match reader.try_read(&mut temp_buf) {
            Ok(0) => {
                println!("Received 0 bytes breaking");
                break;
            }
            Ok(bytes_read) => {
                println!("Received {} bytes", bytes_read);
                handle_read_bytes(bytes_read, &mut temp_buf, &mut context_buf)?
            }

            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        if let Some(message) = message {
            let iso_message = IsoMessage::new(message);

            println!("Message type: {:?}", iso_message.get_type());
        }

        println!("Bytes in context_buf after: {}", context_buf.len());
        println!("Bytes in temp_buf after: {}", temp_buf.len());
    }

    Ok(())
}

fn handle_read_bytes(
    bytes_read: usize,
    temp_buf: &mut [u8],
    context_buf: &mut Vec<u8>,
) -> Result<Option<String>, io::Error> {
    let mut message: Option<String> = None;

    if context_buf.len() == 0 {
        println!("Context Buffer is Empty route");

        let expect_message_length = get_message_length(&temp_buf)?;

        println!(
            "Expected message length in buffer: {}",
            expect_message_length
        );

        if bytes_read >= (expect_message_length as usize + LENGTH_PREFIX_SIZE) {
            println!(
                "{} >= {} + {}",
                bytes_read, expect_message_length, LENGTH_PREFIX_SIZE
            );

            println!(
                "Buffer to convert: {:?}",
                temp_buf[LENGTH_PREFIX_SIZE..(LENGTH_PREFIX_SIZE + expect_message_length as usize)]
                    .to_vec()
            );

            message = Some(
                String::from_utf8_lossy(
                    &temp_buf
                        [LENGTH_PREFIX_SIZE..(LENGTH_PREFIX_SIZE + expect_message_length as usize)],
                )
                .to_string(),
            );

            context_buf.append(
                &mut temp_buf[LENGTH_PREFIX_SIZE + expect_message_length as usize..bytes_read]
                    .to_vec(),
            );
        } else {
            println!(
                "{} != {} + {}",
                bytes_read, expect_message_length, LENGTH_PREFIX_SIZE
            );
            panic!("Not ready for this use case yet");

            // context_buf.append(&mut temp_buf[..bytes_read].to_vec());
        }
    } else {
        let _expect_message_length = get_message_length(&context_buf)?;

        panic!("Not ready for this use case yet");
    }
    Ok(message)
}

fn get_message_length(buf: &[u8]) -> Result<u16, io::Error> {
    (&buf[0..LENGTH_PREFIX_SIZE]).read_u16::<NetworkEndian>()
}
