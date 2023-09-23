// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use reqwest::Client;

once_cell!(req, CLIENT: Client);

module!(
  Reqwest {
    user_agent: String = default_env!("USER_AGENT", "neko.rs"),
  }

  fn init(fw) {
    runtime!(fw, |m| {
      CLIENT.set(Client::builder().user_agent(m.user_agent).build()?)?;
      Ok(None)
    });
  }
);
