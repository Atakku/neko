// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  pub enum SteamUsers {
    UserId.big_integer().primary_key(),
    Username.text(),
    Avatar.text(),
    LastOnline.big_integer(),
  }

  pub enum SteamApps {
    AppId.big_integer().primary_key(),
    AppName.text(),
  }

  pub enum SteamPlaydata {
    PlaydataId.big_integer().primary_key().extra("GENERATED ALWAYS AS IDENTITY"),
    UserId.big_integer().not_null(),
    AppId.big_integer().not_null(),
    Playtime.integer().not_null();
    Self.foreign_key(fk!(SteamUsers, UserId, Cascade, Cascade))
      .foreign_key(fk!(SteamApps, AppId, Cascade, Cascade))
      .index(uk!(UserId, AppId))
  }

  pub enum SteamPlayhist {
    PlaydataId.big_integer().not_null(),
    UtcDay.integer().not_null(),
    Playtime.integer().not_null();
    Self.foreign_key(fk!(SteamPlaydata, PlaydataId, Cascade, Cascade))
      .index(uk!(PlaydataId, UtcDay))
  }
}
