// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use reqwest::Client;

/// Reqwest wrapper module for sharing a configured client
pub struct Reqwest {
  user_agent: String,
}

impl Default for Reqwest {
  fn default() -> Self {
    Self {
      user_agent: default_env!("USER_AGENT", "neko.rs"),
    }
  }
}

init_once!(reqwest, CLIENT: reqwest::Client);

impl Module for Reqwest {
  fn init(&mut self, fw: &mut Framework) -> R {
    fw.runtime.push(|m| {
      let this = m.take::<Self>()?;
      rt_async!({
        CLIENT.set(Client::builder().user_agent(this.user_agent).build()?)?;
        Ok(None)
      })
    });
    Ok(())
  }
}
