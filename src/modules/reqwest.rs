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

macro_rules! api {
  ($base:literal, {
    $(
      fn $fun:ident($endpoint:literal) -> $ty:ty $({
        $($pn:ident:$pt:ty),*$(,)?
      })?;
    )*
  }) => {
    $(pub async fn $fun($($($pn: $pt),*)?) -> crate::core::Res<$ty> {
      let req = format!(concat!($base, $endpoint, $("?",$(stringify!($pn), "={", stringify!($pn), "}&"),*)?), $($($pn=$pn),*)?);
      log::trace!("Sending req to {req}");
      let res = crate::modules::reqwest::req().get(req).send().await?;
      log::trace!("Received status: {}", res.status());
      Ok(res.json::<$ty>().await?)
    })*
  };
}
