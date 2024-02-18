use async_std::prelude::*;

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::marker::Unpin;

pub type ChatError = Box<dyn Error + Send + Sync + 'static>;
pub type ChatResult<T> = Result<T, ChatError>;

pub async fn send_json<O, P>(leaving: &mut O, packet: &P) -> ChatResult<()>
where
    O: async_std::io::Write + Unpin,
    P: Serialize, {
        let mut json = serde_json::to_string(&packet)?;
        json.push('\n');
        leaving.write_all(json.as_bytes()).await?;
        Ok(())
}

pub fn recieve<I, T>(incoming: I) -> impl Stream<Item = ChatResult<T>>
where
    I: async_std::io::BufRead + Unpin,
    T: DeserializeOwned, {
        incoming.lines().map(|line| -> ChatResult<T> {
            let li = line?;
            let msg = serde_json::from_str::<T>(&li)?;
            Ok(msg)
    })
}
