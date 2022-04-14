use iso_message::IsoMessage;
use message_machine::StateMachine;
use std::{net::SocketAddr, sync::Arc};

use tokio::{
    io::{self, AsyncReadExt, WriteHalf},
    net::{TcpListener, TcpStream},
};

mod iso_message;
mod message_helpers;
mod message_machine;

const SOCKET_PORT: u16 = 8006;
pub const LENGTH_PREFIX_SIZE: usize = 2;

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

async fn handle_connection(stream: TcpStream) -> Result<(), io::Error> {
    let (mut reader, writer) = tokio::io::split(stream);
    let writer = Arc::new(writer);

    let mut state_machine = StateMachine::new();
    let mut temp_buf = [0; 4096];

    loop {
        let received_messages = match reader.read(&mut temp_buf).await {
            Ok(0) => {
                println!("Received 0 bytes breaking");
                break;
            }
            Ok(bytes_read) => {
                println!("Received {} bytes", bytes_read);
                state_machine.process(&temp_buf[..bytes_read])
            }

            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        if let Some(messages) = received_messages {
            for message in messages {
                let socket_writer = writer.clone();
                tokio::spawn(async move {
                    handle_message(message, socket_writer).await;
                });
            }
        }
    }

    Ok(())
}

async fn handle_message(_message: IsoMessage, _socket_writer: Arc<WriteHalf<TcpStream>>) {
    // Almost there
    todo!();
}
