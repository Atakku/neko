// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  #[table("neko_users")]
  pub enum Users {
    NekoId, Slug
  }

  #[table("neko_users_discord")]
  pub enum UsersDiscord {
    NekoId, DiscordId
  }

  #[table("neko_users_steam")]
  pub enum UsersSteam {
    NekoId, SteamId
  }

  #[table("neko_users_github")]
  pub enum UsersGithub {
    NekoId, GithubId
  }

  #[table("neko_users_anilist")]
  pub enum UsersAnilist {
    NekoId, AnilistId
  }

  #[table("neko_users_telegram")]
  pub enum UsersTelegram {
    NekoId, TelegramId
  }

  #[table("neko_whitelist_discord")]
  pub enum WhitelistDiscord {
    GuildId
  }
}
