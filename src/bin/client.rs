use async_std::io::BufReader;
use async_std::prelude::*;
use async_std::{task, io, net};
use std::sync::Arc;

use ChatApp::utils::{self, ChatResult};
use ChatApp::{Client, Server};

fn get_value(mut input: &str) -> Option<(&str, &str)> {
    input = input.trim_start();

    if input.is_empty() {
        return None;
    }

    match input.find(char::is_whitespace) {
        Some(whitespace) => Some((&input[0..whitespace], &input[whitespace..])),
        None => Some((input, "")),
    }
}

fn parse_input(line: &str) -> Option<Client> {
    let (input, remainder) = get_value(line)?;

    if input == "join" {
        let (chat, remainder) = get_value(remainder)?;

        if !remainder.trim_start().is_empty() {
            return None;
        }

        return Some(Client::Join {chat_name: Arc::new(chat.to_string()),
        });
    }

    else if input == "post" {
        let (chat, remainder) = get_value(remainder)?;
        let message = remainder.trim_start().to_string();

        return Some(Client::Post {chat_name: Arc::new(chat.to_string()), message: Arc::new(message),
        });
    }

    else {
        println!("Unrecognized input: {:?}", line);
        return None;
    }
}

async fn send(mut send: net::TcpStream) -> ChatResult<()> {
    println!("Options: \njoin CHAT\npost CHAT MESSAGE");

    let mut options = BufReader::new(io::stdin()).lines();
    while let Some(option_result) = options.next().await {
        let opt = option_result?;
        let request = match parse_input(&opt) {
            Some(request) => request,
            None => continue,
        };
        utils::send_json(&mut send, &request).await?;
        send.flush().await?;
    }
    Ok(())
}

async fn messages(server: net::TcpStream) -> ChatResult<()> {
    let buffer = BufReader::new(server);

    let mut stream = utils::recieve(buffer);

    while let Some(message) = stream.next().await {
        match message? {
            Server::Message {chat_name, message} => {
                println!("Chat name: {}\n, Message: {}\n", chat_name, message);
            }
            Server::Error(message) => {
                println!("Error received: {}", message);
            }
        }
    }
    Ok(())
}

fn main() -> ChatResult<()> {
    let addr = std::env::args().nth(1).expect("Address:PORT");

    task::block_on(async {
        let socket = net::TcpStream::connect(addr).await?;
        socket.set_nodelay(true)?;

        let send = send(socket.clone());
        let replies = messages(socket);

        replies.race(send).await?;
        Ok(())
    })
}
