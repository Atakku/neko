use std::collections::HashMap;

use crate::{
  core::{Err, Res},
  modules::{axum::Axum, reqwest::req, sqlx::db},
  plugins::steam::query::{update_playdata, update_users},
};
use askama::Template;
use axum::{
  http::HeaderValue,
  response::{IntoResponse, Redirect, Response},
  routing::get,
  Form, Json,
};
use axum_session::{SessionConfig, SessionLayer, SessionPgSession, SessionPgSessionStore};
use poise::serenity_prelude::{json::json, UserId};
use regex::Regex;
use reqwest::{header, StatusCode};
use sea_query::{Alias, Expr, Func, Iden, InsertStatement, OnConflict, Query, SelectStatement};
use url::Url;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsPage {
  id: i32
}

async fn root(session: SessionPgSession) -> Response {
  let mut res = match session.get::<i32>("neko_id") {
    Some(id) => 
    SettingsPage {id}.render(),
    None => LoginPage.render()
  }.unwrap().into_response();
  res
    .headers_mut()
    .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
  res
}
async fn logout(session: SessionPgSession) -> Response {
  match session.get::<i32>("neko_id") {
    Some(_) => {
      session.destroy();
      Redirect::to("/").into_response()
    }
    None => {
      let mut res = format!("you are not logged in lmao").into_response();
      res
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
      res
    }
  }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RedirForm {
  redirect: Option<String>,
}

async fn login_now(session: SessionPgSession) -> Response {
  match session.get::<i32>("neko_id") {
    Some(_) => Redirect::to("/").into_response(),
    None => Redirect::to("/link/discord").into_response(),
  }
}

once_cell!(sid_regex, REGEX: Regex);
once_cell!(mc_regex, MCNAMEREGEX: Regex);

#[derive(Default)]
pub struct Gwaaa;

impl crate::core::Module for Gwaaa {
  async fn init(&mut self, fw: &mut crate::core::Framework) -> crate::core::R {
    {
      fw.req_module::<crate::modules::reqwest::Reqwest>().await?;
      fw.req_module::<crate::modules::sqlx::Postgres>().await?;
      REGEX.set(Regex::new(
        "^https://steamcommunity.com/openid/id/([0-9]{17})$",
      )?)?;
      MCNAMEREGEX.set(Regex::new(
        "[^a-zA-Z0-9_].",
      )?)?;

      let axum = fw.req_module::<Axum>().await?;

      axum.routes.push(|r| {
        Box::pin(async move {
          let session_config = SessionConfig::default().with_table_name("neko_users_sessions");
          let session_store = SessionPgSessionStore::new(Some(db().clone().into()), session_config)
            .await
            .unwrap();

          Ok(
            r.route("/", get(root))
              .route("/login", get(login_now))
              .route("/logout", get(logout))
              .route("/whitelist", get(whitelist))
              .route("/callback/anilist", get(callback_anilist))
              .route("/callback/github", get(callback_github))
              .route("/callback/steam", get(callback_steam))
              .route("/callback/discord", get(callback_discord))
              .route("/callback/minecraft", get(callback_minecraft))
              .route("/link/anilist", get(link_anilist))
              .route("/link/github", get(link_github))
              .route("/link/steam", get(link_steam))
              .route("/link/discord", get(link_discord))
              .route("/link/minecraft", get(link_minecraft))
              .layer(SessionLayer::new(session_store))
              .route("/metrics", get(metrics)),
          )
        })
      });
    }
    Ok(())
  }
}

async fn metrics() -> String {
  let mut output = String::new();

  use crate::plugins::steam::schema::*;

  let mut qb = Query::select();
  qb.from(Apps::Table);
  qb.from(Users::Table);
  qb.from(Playdata::Table);
  qb.and_where(ex_col!(Playdata, AppId).equals(col!(Apps, Id)));
  qb.and_where(ex_col!(Playdata, UserId).equals(col!(Users, Id)));
  qb.column(col!(Users, Name));
  qb.expr_as(
    Func::sum(ex_col!(Playdata, Playtime)),
    Alias::new("sum_count"),
  );
  qb.group_by_col(col!(Users, Id));
  match fetch_all!(&qb, (String, i64)) {
    Ok(data) => {
      for (u, p) in data {
        output += &format!("steam_user_summary{{user=\"{u}\"}} {p}\n");
      }
    }
    Result::Err(err) => log::warn!("{}", err),
  }

  output
}

struct GenericError(Err);

impl From<Err> for GenericError {
  fn from(value: Err) -> Self {
    Self(value)
  }
}

impl IntoResponse for GenericError {
  fn into_response(self) -> Response {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "error": self.0.to_string(),
      })),
    )
      .into_response()
  }
}

async fn link_steam() -> axum::response::Result<Response> {
  let mut redirect_url = Url::parse("https://steamcommunity.com/openid/login").unwrap();
  redirect_url.set_query(Some(
    &serde_urlencoded::to_string(&RedirectForm {
      ns: "http://specs.openid.net/auth/2.0",
      identity: "http://specs.openid.net/auth/2.0/identifier_select",
      claimed_id: "http://specs.openid.net/auth/2.0/identifier_select",
      mode: "checkid_setup",
      realm: root_domain().await,
      return_to: &format!("{}/callback/steam", root_domain().await),
    })
    .unwrap(),
  ));
  Ok(Redirect::to(redirect_url.as_str()).into_response())
}

// who needs actual error handling tbh
async fn callback_steam(
  session: SessionPgSession,
  Form(cb): Form<VerifyForm>
) -> axum::response::Result<Response> {
  let Some(id) = session.get::<i32>("neko_id") else {
    return Ok(Redirect::to("/login").into_response());
  };
  let mut validate = cb;
  validate.mode = "check_authentication".to_owned();
  let form_str = serde_urlencoded::to_string(&validate).unwrap();

  let response = req()
    .post("https://steamcommunity.com/openid/login")
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(form_str)
    .send()
    .await
    .unwrap()
    .text()
    .await
    .unwrap();

  let is_valid = response.split('\n').any(|line| line == "is_valid:true");
  if !is_valid {
    return Err("NOT VALID GWAAAA".into());
  }

  let captures = sid_regex().captures(&validate.claimed_id).unwrap();
  let steam_id = captures.get(1).unwrap().as_str().parse::<i64>().unwrap();
  use crate::plugins::neko::schema::UsersSteam::*;
  let mut qb = InsertStatement::new();
  qb.into_table(Table);
  qb.columns([NekoId, SteamId]);
  qb.values([id.into(), steam_id.into()]).unwrap();
  qb.on_conflict(OnConflict::column(SteamId).update_column(NekoId).to_owned());
  execute!(&qb).unwrap();

  let c = vec![(steam_id,)];
  update_users(&c).await.unwrap();
  update_playdata(&c).await.unwrap();
  Ok(Redirect::to("/").into_response())
}

once_cell!(root_domain, ROOT_DOMAIN: String, {expect_env!("ROOT_DOMAIN")});

once_cell!(oauth_discord_id, OAUTH_DISCORD_ID: String, {expect_env!("OAUTH_DISCORD_ID")});
once_cell!(oauth_discord_secret, OAUTH_DISCORD_SECRET: String, {expect_env!("OAUTH_DISCORD_SECRET")});
once_cell!(redirect_discord, REDIRECT_DISCORD: String, {
  let cb = format!("{}/callback/discord", root_domain().await);
  format!("https://discord.com/oauth2/authorize\
  ?client_id={}&redirect_uri={}&response_type=code\
  &scope=identify&prompt=consent&state=todo",
  oauth_discord_id().await, urlencoding::encode(&cb))
});

once_cell!(oauth_github_id, OAUTH_GITHUB_ID: String, {expect_env!("OAUTH_GITHUB_ID")});
once_cell!(oauth_github_secret, OAUTH_GITHUB_SECRET: String, {expect_env!("OAUTH_GITHUB_SECRET")});

once_cell!(redirect_github, REDIRECT_GITHUB: String, {
  let cb = format!("{}/callback/github", root_domain().await);
  format!("https://github.com/login/oauth/authorize\
  ?client_id={}&redirect_uri={}&response_type=code\
  &allow_signup=false&state=todo", oauth_github_id().await,
  urlencoding::encode(&cb))
});

once_cell!(tokenreq_github, TOKENREQ_GITHUB: String, {
  let cb = format!("{}/callback/github", root_domain().await);
  format!("https://github.com/login/oauth/access_token\
  ?client_id={}&client_secret={}&redirect_uri={}",
  oauth_github_id().await, oauth_github_secret().await,
  urlencoding::encode(&cb))
});

once_cell!(oauth_anilist_id, OAUTH_ANILIST_ID: String, {expect_env!("OAUTH_ANILIST_ID")});
once_cell!(oauth_anilist_secret, OAUTH_ANILIST_SECRET: String, {expect_env!("OAUTH_ANILIST_SECRET")});

once_cell!(oauth_minecraft_id, OAUTH_MINECRAFT_ID: String, {expect_env!("OAUTH_MINECRAFT_ID")});
once_cell!(oauth_minecraft_secret, OAUTH_MINECRAFT_SECRET: String, {expect_env!("OAUTH_MINECRAFT_SECRET")});

once_cell!(redirect_anilist, REDIRECT_ANILIST: String, {
  let cb = format!("{}/callback/anilist", root_domain().await);
  format!("https://anilist.co/api/v2/oauth/authorize\
  ?client_id={}&redirect_uri={}&response_type=code&state=todo", oauth_anilist_id().await,
  urlencoding::encode(&cb))
});

once_cell!(redirect_minecraft, REDIRECT_MINECRAFT: String, {
  let cb = format!("{}/callback/minecraft", root_domain().await);
  format!("https://mc-auth.com/oAuth2/authorize\
  ?client_id=3551875741534651542&redirect_uri={}&response_type=code&scope=profile&state=todo",
  urlencoding::encode(&cb))
});

async fn link_anilist() -> axum::response::Result<Response> {
  Ok(Redirect::to(redirect_anilist().await).into_response())
}

async fn link_minecraft() -> axum::response::Result<Response> {
  Ok(Redirect::to(redirect_minecraft().await).into_response())
}

async fn callback_minecraft(
  session: SessionPgSession,
  Form(cb): Form<AuthorizationCallback>,
) -> axum::response::Result<Response> {
  let Some(id) = session.get::<i32>("neko_id") else {
    return Ok(Redirect::to("/login").into_response());
  };
  if cb.state != "todo" {
    return Ok(StatusCode::IM_A_TEAPOT.into_response());
  }
  let form_str = &DiscordTokenReq {
    client_id: &"3551875741534651542",
    client_secret: &expect_env!("OAUTH_MINECRAFT_SECRET"),
    grant_type: &"authorization_code",
    code: &cb.code,
    redirect_uri: &format!("{}/callback/minecraft", root_domain().await),
  };

  let res = req()
    .post("https://mc-auth.com/oAuth2/token")
    .json(form_str)
    .send()
    .await.unwrap();

  let Ok(response) =  res.json::<MCTokenRes>().await else {
    return Err("NOT VALID GWAAAA".into());
  };

  use UsersMinecraft::*;
  let mut qb = InsertStatement::new();
  qb.into_table(Table);
  qb.columns([NekoId, McUuid]);
  qb.values([id.into(), response.data.uuid.into()]).unwrap();
  qb.on_conflict(OnConflict::column(NekoId).do_nothing().to_owned());
  execute!(&qb).unwrap();

  Ok(Redirect::to("/").into_response())
}

async fn whitelist(Form(q): Form<Bruh>) -> axum::response::Result<Response> {
  println!("uuid: {}", q.uuid);
  let mut qb = SelectStatement::new();
  qb.from(UsersMinecraft::Table);
  qb.and_where(Expr::col(UsersMinecraft::McUuid).eq(q.uuid));
  
  use super::neko::schema::UsersDiscord;
  qb.from(UsersDiscord::Table);
  qb.and_where(ex_col!(UsersMinecraft, NekoId).equals(col!(UsersDiscord, NekoId)));
  
  use super::discord::schema::Members;
  qb.from(Members::Table);
  qb.and_where(ex_col!(Members, GuildId).eq(1404602275401568347_i64));
  qb.and_where(ex_col!(Members, UserId).equals(col!(UsersDiscord, DiscordId)));

  use super::discord::schema::Users;
  qb.from(Users::Table);
  qb.and_where(ex_col!(Users, Id).equals(col!(UsersDiscord, DiscordId)));
  qb.column(col!(Members, Nick));
  qb.column(col!(Users, Name));
  qb.column(col!(Users, Id));

  Ok(match fetch_one!(&qb, (Option<String>, String, i64)) {
    Ok((nick, name, id)) => {
      let mut fancy = nick.map(|n| mc_regex().replace_all(n.replace(" ", "_").replace("__", "_").replace("..", ".").as_str(), "").to_string()).unwrap_or(name.to_string());
      if fancy.len() < 2 {
        fancy = name;
      }
      format!("{fancy}\n{id}").into_response()
    },
    Result::Err(err) => {
      log::error!("Error when getting whitelist: {err}");
      StatusCode::NOT_FOUND.into_response()
    },
  })
}

#[derive(Iden)]
#[iden(rename = "neko_users_minecraft")]
pub enum UsersMinecraft {
  Table,
  NekoId,
  McUuid,
}

pub async fn get_mc_users() -> Res<Vec<UserId>> {
  let mut qb = Query::select();
  
  use super::neko::schema::UsersDiscord;
  use super::gwaaa::UsersMinecraft;
  qb.from(UsersMinecraft::Table);
  qb.from(UsersDiscord::Table);
  qb.and_where(ex_col!(UsersMinecraft, NekoId).equals(col!(UsersDiscord, NekoId)));
  qb.column(col!(UsersDiscord, DiscordId));

  Ok(
    fetch_all!(&qb, (i64,))?
      .into_iter()
      .map(|r| UserId(r.0 as u64))
      .collect(),
  )
}

async fn link_discord() -> axum::response::Result<Response> {
  Ok(Redirect::to(redirect_discord().await).into_response())
}

async fn link_github() -> axum::response::Result<Response> {
  Ok(Redirect::to(redirect_github().await).into_response())
}

async fn callback_discord(
  session: SessionPgSession,
  Form(cb): Form<AuthorizationCallback>,
) -> axum::response::Result<Response> {
  if cb.state != "todo" {
    return Ok(StatusCode::IM_A_TEAPOT.into_response());
  }
  let form_str = serde_urlencoded::to_string(&DiscordTokenReq {
    client_id: &"1064379551318278204",
    client_secret: &expect_env!("OAUTH_DISCORD_SECRET"),
    grant_type: &"authorization_code",
    code: &cb.code,
    redirect_uri: &format!("{}/callback/discord", root_domain().await),
  })
  .unwrap();

  let response = req()
    .post("https://discord.com/api/v10/oauth2/token")
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(form_str)
    .send()
    .await
    .unwrap()
    .json::<TokenRes>()
    .await
    .unwrap();

  let response = req()
    .get("https://discord.com/api/v10/oauth2/@me")
    .header("Content-Type", "application/json")
    .bearer_auth(response.access_token)
    .send()
    .await
    .unwrap()
    .json::<DiscordAuthRes>()
    .await
    .unwrap();

  if let None = response.user {
    return Err("NOT VALID GWAAAA".into());
  }

  let did = response.user.unwrap().id.parse::<i64>().unwrap();

  let id = match session.get::<i32>("neko_id") {
    None => {
      use crate::plugins::neko::schema::UsersDiscord;
      let mut qb = SelectStatement::new();
      qb.from(UsersDiscord::Table);
      qb.column(UsersDiscord::NekoId);
      qb.and_where(ex_col!(UsersDiscord, DiscordId).eq(did));
      let newid = match fetch_optional!(&qb, (i32,)).unwrap() {
        Some(id) => id.0,
        None => {
          use crate::plugins::neko::schema::Users::*;
          let mut qb = InsertStatement::new();
          qb.into_table(Table);
          qb.columns([Slug]);
          qb.values([Option::<String>::None.into()]).unwrap();
          qb.returning(Query::returning().columns([Id]));
          fetch_one!(&qb, (i32,)).unwrap().0
        }
      };
      session.set("neko_id", newid);
      newid
    }
    Some(id) => id,
  };
  use crate::plugins::neko::schema::UsersDiscord::*;
  let mut qb = InsertStatement::new();
  qb.into_table(Table);
  qb.columns([NekoId, DiscordId]);
  qb.values([id.into(), did.into()]).unwrap();
  qb.on_conflict(
    OnConflict::column(DiscordId)
      .update_column(NekoId)
      .to_owned(),
  );
  execute!(&qb).unwrap();
  Ok(Redirect::to("/").into_response())
}

async fn callback_github(
  session: SessionPgSession,
  Form(cb): Form<AuthorizationCallback>,
) -> axum::response::Result<Response> {
  let Some(id) = session.get::<i32>("neko_id") else {
    return Ok(Redirect::to("/login").into_response());
  };
  if cb.state != "todo" {
    return Ok(StatusCode::IM_A_TEAPOT.into_response());
  }

  let response = req()
    .post(format!("{}&code={}", tokenreq_github().await, cb.code))
    .header("Content-Type", "application/x-www-form-urlencoded")
    .header("Accept", "application/json")
    .send()
    .await
    .unwrap()
    .json::<TokenRes>()
    .await
    .unwrap();

  let response = req()
    .get("https://api.github.com/user")
    .header("Content-Type", "application/json")
    .header("X-GitHub-Api-Version", "2022-11-28")
    .header("Accept", "application/vnd.github+json")
    .bearer_auth(response.access_token)
    .send()
    .await
    .unwrap()
    .json::<GithubRes>()
    .await
    .unwrap();

  let gid = response.id;
  use crate::plugins::neko::schema::UsersGithub::*;
  let mut qb = InsertStatement::new();
  qb.into_table(Table);
  qb.columns([NekoId, GithubId]);
  qb.values([id.into(), gid.into()]).unwrap();
  qb.on_conflict(
    OnConflict::column(GithubId)
      .update_column(GithubId)
      .to_owned(),
  );
  execute!(&qb).unwrap();
  Ok(Redirect::to("/").into_response())
}

async fn callback_anilist(
  session: SessionPgSession,
  Form(cb): Form<AuthorizationCallback>,
) -> axum::response::Result<Response> {
  let Some(id) = session.get::<i32>("neko_id") else {
    return Ok(Redirect::to("/login").into_response());
  };
  if cb.state != "todo" {
    return Ok(StatusCode::IM_A_TEAPOT.into_response());
  }
  let response = req()
    .post("https://anilist.co/api/v2/oauth/token")
    .header("Content-Type", "application/json")
    .header("Accept", "application/json")
    .json(&json!({
      "client_id": oauth_anilist_id().await,
      "client_secret": oauth_anilist_secret().await,
      "grant_type": "authorization_code",
      "redirect_uri": format!("{}/callback/anilist", root_domain().await),
      "code": cb.code
    }))
    .send()
    .await
    .unwrap()
    .json::<TokenRes>()
    .await
    .unwrap();

  let response = req()
    .post("https://graphql.anilist.co")
    .header("Content-Type", "application/json")
    .header("Accept", "application/json")
    .bearer_auth(response.access_token)
    .json(&json!({
      "query": "{Viewer{id}}"
    }))
    .send()
    .await
    .unwrap()
    .json::<AnilistRes>()
    .await
    .unwrap();
  let gid = response.data.viewer.id;
  use crate::plugins::neko::schema::UsersAnilist::*;
  let mut qb = InsertStatement::new();
  qb.into_table(Table);
  qb.columns([NekoId, AnilistId]);
  qb.values([id.into(), gid.into()]).unwrap();
  qb.on_conflict(
    OnConflict::column(AnilistId)
      .update_column(AnilistId)
      .to_owned(),
  );
  execute!(&qb).unwrap();
  Ok(Redirect::to("/").into_response())
}

#[derive(serde::Serialize)]
struct RedirectDiscord<'a> {
  response_type: &'static str,
  client_id: &'static str,
  scope: &'static str,
  state: &'a str,
  redirect_uri: String,
  prompt: &'a str,
}

#[derive(serde::Deserialize)]
struct AuthorizationCallback {
  code: String,
  state: String,
}
#[derive(serde::Deserialize)]
struct Bruh {
  uuid: Uuid
}


#[derive(serde::Serialize, Debug)]
struct DiscordTokenReq<'a> {
  client_id: &'a str,
  client_secret: &'a str,
  grant_type: &'a str,
  code: &'a str,
  redirect_uri: &'a str,
}

#[derive(serde::Deserialize)]
struct MCTokenRes {
  data: MCData,
}
#[derive(serde::Deserialize)]
struct MCData {
  uuid: Uuid,
}
#[derive(serde::Deserialize)]
struct TokenRes {
  access_token: String,
}
#[derive(serde::Deserialize)]
struct DiscordAuthRes {
  user: Option<DiscordUser>,
}

#[derive(serde::Deserialize)]
struct AnilistRes {
  data: AnilistData,
}
#[derive(serde::Deserialize)]
struct AnilistData {
  #[serde(rename = "Viewer")]
  viewer: GithubRes,
}
#[derive(serde::Deserialize)]
struct GithubRes {
  id: i64,
}
#[derive(serde::Deserialize)]
struct DiscordUser {
  id: String,
}

#[derive(serde::Serialize)]
struct RedirectForm<'a> {
  #[serde(rename = "openid.ns")]
  ns: &'static str,
  #[serde(rename = "openid.identity")]
  identity: &'static str,
  #[serde(rename = "openid.claimed_id")]
  claimed_id: &'static str,
  #[serde(rename = "openid.mode")]
  mode: &'static str,
  #[serde(rename = "openid.return_to")]
  return_to: &'a str,
  #[serde(rename = "openid.realm")]
  realm: &'a str,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VerifyForm {
  #[serde(rename = "openid.ns")]
  ns: String,
  #[serde(rename = "openid.mode")]
  mode: String,
  #[serde(rename = "openid.op_endpoint")]
  op_endpoint: String,
  #[serde(rename = "openid.claimed_id")]
  claimed_id: String,
  #[serde(rename = "openid.identity")]
  identity: Option<String>,
  #[serde(rename = "openid.return_to")]
  return_to: String,
  #[serde(rename = "openid.response_nonce")]
  response_nonce: String,
  #[serde(rename = "openid.invalidate_handle")]
  invalidate_handle: Option<String>,
  #[serde(rename = "openid.assoc_handle")]
  assoc_handle: String,
  #[serde(rename = "openid.signed")]
  signed: String,
  #[serde(rename = "openid.sig")]
  sig: String,
}
