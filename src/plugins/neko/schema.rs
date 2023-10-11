// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  #[table("neko_users")]
  pub enum NekoUsers {
    NekoId, Slug
  }

  #[table("neko_users_discord")]
  pub enum NekoUsersDiscord {
    NekoId, DiscordId
  }

  #[table("neko_users_steam")]
  pub enum NekoUsersSteam {
    NekoId, SteamId
  }

  #[table("neko_users_github")]
  pub enum NekoUsersGithub {
    NekoId, GithubId
  }

  #[table("neko_users_anilist")]
  pub enum NekoUsersAnilist {
    NekoId, AnilistId
  }

  #[table("neko_users_telegram")]
  pub enum NekoUsersTelegram {
    NekoId, TelegramId
  }

  #[table("neko_whitelist_discord")]
  pub enum NekoWhitelistDiscord {
    GuildId
  }
}
