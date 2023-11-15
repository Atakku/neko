// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  pub enum NekoUsers {
    NekoId.big_integer().extra("GENERATED ALWAYS AS IDENTITY"), 
    Slug.char_len(32).unique_key()
  }

  pub enum NekoUsersDiscord {
    NekoId.big_integer().not_null(),
    DiscordId.big_integer().primary_key();
    Self.foreign_key(fk!(NekoUsers, NekoId, Cascade, Cascade))
  }

  pub enum NekoUsersSteam {
    NekoId.big_integer().not_null(),
    SteamId.big_integer().primary_key();
    Self.foreign_key(fk!(NekoUsers, NekoId, Cascade, Cascade))
  }

  pub enum NekoUsersGithub {
    NekoId.big_integer().not_null(),
    GithubId.big_integer().primary_key();
    Self.foreign_key(fk!(NekoUsers, NekoId, Cascade, Cascade))
  }

  pub enum NekoUsersAnilist {
    NekoId.big_integer().not_null(),
    AnilistId.big_integer().primary_key();
    Self.foreign_key(fk!(NekoUsers, NekoId, Cascade, Cascade))
  }

  pub enum NekoUsersTelegram {
    NekoId.big_integer().not_null(),
    TelegramId.big_integer().primary_key();
    Self.foreign_key(fk!(NekoUsers, NekoId, Cascade, Cascade))
  }

  pub enum NekoWhitelistDiscord {
    GuildId.big_integer().primary_key()
  }
}
