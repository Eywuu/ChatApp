use async_std::{net, task};
use ChatApp::utils::ChatResult;
use std::sync::Arc;
use async_std::prelude::*;

mod connection;
mod chats_map;
mod chats;

use connection::handle;

fn log_error(result: ChatResult<()>) {
    if let Err(error) = result {
        println!("Error: {}", error);
    }
}

fn main() -> ChatResult<()> {
    let addr = std::env::args().nth(1).expect("server ADDRESS");

    let chat_table = Arc::new(chats_map::ChatTracker::new());

    async_std::task::block_on(async {
        let listener = net::TcpListener::bind(addr).await?;

        let mut new_connection = listener.incoming();
        while let Some(socket_result) = new_connection.next().await {
            let socket = socket_result?;
            let chats = chat_table.clone();

            task::spawn(async {
                log_error(handle(socket, chats).await);
            });
        }
        Ok(())
    })
}