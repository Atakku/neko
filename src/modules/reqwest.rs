// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use reqwest::Client;

once_cell!(req, CLIENT: Client);

module! {
  Reqwest {
    user_agent: String = env!("USER_AGENT", "neko.rs"),
  }

  fn init(fw) {
    rt!(fw, |reqwest| {
      CLIENT.set(Client::builder().user_agent(reqwest.user_agent).build()?)?;
      Ok(None)
    });
  }
}
