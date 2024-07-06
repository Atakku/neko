// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use reqwest::Client;

once_cell!(req, CLIENT: Client);

pub struct Reqwest {
  pub user_agent: String,
}

impl Default for Reqwest {
  fn default() -> Self {
    Self {
      user_agent: default_env!("USER_AGENT", "neko.rs"),
    }
  }
}

impl crate::core::Module for Reqwest {
  async fn init(&mut self, fw: &mut crate::core::Framework) -> crate::core::R {
    {
      runtime!(fw, |m| {
        CLIENT.set(Client::builder().user_agent(m.user_agent).build()?)?;
        Ok(None)
      });
    }
    Ok(())
  }
}
