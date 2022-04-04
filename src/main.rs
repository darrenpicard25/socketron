use byteorder::{BigEndian, NetworkEndian, ReadBytesExt}; // 1.2.7
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
    let (reader, writer) = stream.split();

    let mut context_buf: Vec<u8> = Vec::with_capacity(3_418_usize + 2); // Based on ISO 8583 spec
    let mut temp_buf: Vec<u8> = Vec::with_capacity(3_418_usize + 2);

    loop {
        reader.readable().await?;
        let message = match reader.try_read(&mut temp_buf) {
            Ok(0) => {
                println!("Received 0 bytes breaking");
                break;
            }
            Ok(_) => handle_read_bytes(&mut temp_buf, &mut context_buf)?,

            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        if let Some(message) = message {
            println!("Gotta full message: \n{}", message);
        }

        println!("Bytes in context_buf after: {}", context_buf.len());
        println!("Bytes in temp_buf after: {}", temp_buf.len());
    }

    Ok(())
}

fn handle_read_bytes(
    temp_buf: &mut Vec<u8>,
    context_buf: &mut Vec<u8>,
) -> Result<Option<String>, io::Error> {
    let mut message: Option<String> = None;

    if context_buf.len() == 0 {
        let expect_message_length = get_message_length(&temp_buf)?;

        if temp_buf.len() >= (expect_message_length as usize + LENGTH_PREFIX_SIZE) {
            message = Some(
                String::from_utf8(
                    temp_buf[LENGTH_PREFIX_SIZE..expect_message_length as usize].to_vec(),
                )
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid utf-8"))?,
            );

            context_buf.append(
                &mut temp_buf[LENGTH_PREFIX_SIZE + expect_message_length as usize..].to_vec(),
            );
            temp_buf.clear();
        } else {
            context_buf.append(temp_buf);
        }
    } else {
        let expect_message_length = get_message_length(&context_buf)?;

        panic!("Not ready for this use case yet");
    }
    Ok(message)
}

fn get_message_length(buf: &[u8]) -> Result<u16, io::Error> {
    (&buf[0..LENGTH_PREFIX_SIZE]).read_u16::<NetworkEndian>()
}
