use crate::constants;
use crate::helpers;
use crate::helpers::upsert_user;
use std::io::{self, Write};
use tokio::runtime::Runtime;
use url::form_urlencoded;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{Rng, distr::Alphanumeric};
use reqwest::Client;
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

use serde::{Deserialize, Deserializer, Serialize};

fn state_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
	D: Deserializer<'de>,
{
	#[derive(Deserialize)]
	#[serde(untagged)]
	enum StatePossibility {
		String(String),
		Bool(bool),
	}

	match StatePossibility::deserialize(deserializer)? {
		StatePossibility::String(s) => Ok(match s.as_str() {
			"ACTIVE" => true,
			"INACTIVE" => false,
			_ => false, // or handle error as needed
		}),
		StatePossibility::Bool(b) => Ok(b),
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MinecraftProfileInfo {
	pub id: String,
	#[serde(deserialize_with = "state_to_bool")]
	pub state: bool,
	pub url: String,
	pub alias: Option<String>,
	pub variant: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
	pub access_token: String,
	pub refresh_token: String,
	pub id: String,
	pub name: String,
	pub skins: Vec<MinecraftProfileInfo>,
	pub capes: Vec<MinecraftProfileInfo>,
}

pub struct AuthParams {
	pub client_id: String,
	pub response_type: String,
	pub redirect_uri: String,
	pub response_mode: String,
	pub scope: String,
	pub state: Option<String>,
	pub code_challenge: String,
	pub code_challenge_method: &'static str,
}

pub struct AuthParamsIter<'a> {
	params: &'a AuthParams,
	index: usize,
}

impl<'a> IntoIterator for &'a AuthParams {
	type Item = (&'static str, String);
	type IntoIter = AuthParamsIter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		AuthParamsIter {
			params: self,
			index: 0,
		}
	}
}

impl<'a> Iterator for AuthParamsIter<'a> {
	type Item = (&'static str, String);

	fn next(&mut self) -> Option<Self::Item> {
		let res = match self.index {
			0 => Some(("client_id", self.params.client_id.clone())),
			1 => Some(("response_type", self.params.response_type.clone())),
			2 => Some(("redirect_uri", self.params.redirect_uri.clone())),
			3 => Some(("response_mode", self.params.response_mode.clone())),
			4 => Some(("scope", self.params.scope.clone())),
			5 => self.params.state.clone().map(|s| ("state", s)),
			6 => Some(("code_challenge", self.params.code_challenge.clone())),
			7 => Some((
				"code_challenge_method",
				self.params.code_challenge_method.to_string(),
			)),
			_ => None,
		};
		self.index += 1;
		res
	}
}

#[derive(Debug, Deserialize)]
pub struct McAuthResponse {
	#[allow(dead_code)]
	pub username: String,
	#[allow(dead_code)]
	pub roles: Vec<String>,
	pub access_token: String,
	#[allow(dead_code)]
	pub token_type: String,
	#[allow(dead_code)]
	pub expires_in: i64,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct XSTSTokProperties<'a> {
	pub SandboxId: &'a str,
	pub UserTokens: Vec<&'a str>,
}
#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct XSTSTokRequest<'a> {
	pub Properties: XSTSTokProperties<'a>,
	pub RelyingParty: &'a str,
	pub TokenType: &'a str,
}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct XSTSResponse<'a> {
	#[allow(dead_code)]
	IssueInstant: &'a str,
	#[allow(dead_code)]
	NotAfter: &'a str,
	pub Token: &'a str,
	#[allow(dead_code)]
	DisplayClaims: DisplayClaims,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct XblAuthResponse {
	#[allow(dead_code)]
	pub IssueInstant: String,
	#[allow(dead_code)]
	pub NotAfter: String,
	pub Token: String,
	pub DisplayClaims: DisplayClaims,
}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct DisplayClaims {
	pub xui: Vec<XuiClaim>,
}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct XuiClaim {
	pub uhs: String,
}
#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct XblAuthRequest<'a> {
	pub Properties: XblProperties<'a>,
	pub RelyingParty: &'a str,
	pub TokenType: &'a str,
}
#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct XblProperties<'a> {
	pub AuthMethod: &'a str,
	pub SiteName: &'a str,
	pub RpsTicket: &'a str,
}

#[derive(Debug, Serialize)]
pub struct AuthTokenParameters {
	pub client_id: String,
	pub scope: String,
	pub code: String,
	pub redirect_uri: String,
	pub grant_type: String,
	pub code_verifier: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
	pub access_token: String,
	#[allow(dead_code)]
	pub expires_in: Option<u64>,
	pub refresh_token: Option<String>,
	#[allow(dead_code)]
	pub token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftProfileResponse {
	pub id: String,
	pub name: String,
	pub skins: Vec<MinecraftProfileInfo>,
	pub capes: Vec<MinecraftProfileInfo>,
}

/// Generates the PKCE code challenge and code verifier.
fn generate_pkce_data() -> (String, String, &'static str) {
	let code_verifier: String = rand::rng()
		.sample_iter(&Alphanumeric)
		.map(char::from)
		.take(128)
		.collect();

	let digest = Sha256::digest(code_verifier.as_bytes());

	let code_challenge = URL_SAFE_NO_PAD.encode(digest);

	let code_challenge_method = "S256";
	(code_verifier, code_challenge, code_challenge_method)
}

pub fn generate_state() -> String {
	Uuid::new_v4().simple().to_string()
}

pub fn get_secure_login_data(
	client_id: &str,
	redirect_url: &str,
	mut state: Option<String>,
) -> (
	url::Url,
	std::option::Option<std::string::String>,
	std::string::String,
) {
	let (code_verifier, code_challenge, code_challenge_method) = generate_pkce_data();

	if state.is_none() {
		state = Some(generate_state())
	}

	let parameters: AuthParams = AuthParams {
		client_id: client_id.into(),
		response_type: "code".into(),
		redirect_uri: redirect_url.into(),
		response_mode: "query".into(),
		scope: "XboxLive.signin offline_access".into(),
		state: state.clone(),
		code_challenge: code_challenge,
		code_challenge_method: code_challenge_method,
	};

	let url = Url::parse_with_params(
		"https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize",
		&parameters,
	)
	.unwrap();

	return (url, state, code_verifier);
}

pub async fn get_auth_token(
	client_id: &str,
	redirect_url: &str,
	auth_code: String,
	code_verifier: String,
) -> Result<TokenResponse, Box<dyn std::error::Error>> {
	let parameters = AuthTokenParameters {
		code: auth_code.clone(),
		scope: "XboxLive.signin offline_access".to_string(),
		client_id: client_id.to_string(),
		grant_type: "authorization_code".to_string(),
		redirect_uri: redirect_url.to_string(),
		code_verifier,
	};

	let client = Client::new();
	let raw_resp = client
		.post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
		.form(&parameters)
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;

	let resp = serde_json::from_str::<TokenResponse>(&raw_resp).unwrap();

	Ok(resp)
}

pub async fn authenticate_with_xbl(
	ms_access_token: &str,
) -> Result<(String /* xbl_token */, String /* user_hash */), Box<dyn std::error::Error>> {
	let rps_ticket: &str = &format!("d={}", ms_access_token);
	let req_body = XblAuthRequest {
		Properties: XblProperties {
			AuthMethod: "RPS",
			SiteName: "user.auth.xboxlive.com",
			RpsTicket: rps_ticket,
		},
		RelyingParty: "http://auth.xboxlive.com",
		TokenType: "JWT",
	};

	let client = Client::new();
	let raw_resp = client
		.post("https://user.auth.xboxlive.com/user/authenticate")
		.body(serde_json::to_string(&req_body).unwrap())
		.header("Content-Type", "application/json")
		.header("Accept", "application/json")
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;
	let resp = serde_json::from_str::<XblAuthResponse>(&raw_resp).unwrap();

	let token = resp.Token;
	let user_hash = resp
		.DisplayClaims
		.xui
		.get(0)
		.ok_or("Missing xui/uhs claim")?
		.uhs
		.clone();

	Ok((token, user_hash))
}

pub async fn get_xsts_token(xbl_token: String) -> Result<String, Box<dyn std::error::Error>> {
	let req_body = XSTSTokRequest {
		Properties: XSTSTokProperties {
			SandboxId: "RETAIL",
			UserTokens: vec![&xbl_token],
		},
		RelyingParty: "rp://api.minecraftservices.com/",
		TokenType: "JWT",
	};

	let client = Client::new();
	let raw_resp = client
		.post("https://xsts.auth.xboxlive.com/xsts/authorize")
		.body(serde_json::to_string(&req_body).unwrap())
		.header("Content-Type", "application/json")
		.header("Accept", "application/json")
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;
	let resp = serde_json::from_str::<XSTSResponse>(&raw_resp).unwrap();
	let token = resp.Token.to_string();

	Ok(token)
}

pub async fn minecraft_services_auth(
	xsts_token: String,
	xbl_userhash: String,
) -> Result<McAuthResponse, Box<dyn std::error::Error>> {
	let client = Client::new();
	let raw_resp = client
		.post("https://api.minecraftservices.com/authentication/login_with_xbox")
		.body(format!(
			"{}\"identityToken\": \"x={};{}\"{}",
			"{", xbl_userhash, xsts_token, "}"
		))
		.header("Content-Type", "application/json")
		.header("Accept", "application/json")
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;
	let resp = serde_json::from_str::<McAuthResponse>(&raw_resp).unwrap();

	Ok(resp)
}

pub async fn get_minecraft_profile(
	token: String,
	refresh_token: String,
) -> Result<User, Box<dyn std::error::Error>> {
	let client = Client::new();

	let raw_resp = client
		.get("https://api.minecraftservices.com/minecraft/profile")
		.header("Authorization", format!("Bearer {}", token))
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;

	let resp = serde_json::from_str::<MinecraftProfileResponse>(&raw_resp).unwrap();

	Ok(User {
		id: resp.id,
		name: resp.name,
		skins: resp.skins,
		capes: resp.capes,
		access_token: token,
		refresh_token: refresh_token,
	})
}

pub async fn complete_login(
	client_id: &str,
	redirect_url: &str,
	auth_code: String,
	code_verifier: String,
) -> User {
	let token_response = get_auth_token(client_id, redirect_url, auth_code, code_verifier)
		.await
		.unwrap();
	let token = token_response.access_token;
	let refresh_token = token_response.refresh_token;

	let xbl_request = authenticate_with_xbl(&token).await.unwrap();

	let xsts_token = get_xsts_token(xbl_request.0).await.unwrap();

	let mc_auth = minecraft_services_auth(xsts_token, xbl_request.1)
		.await
		.unwrap();

	return get_minecraft_profile(mc_auth.access_token, refresh_token.unwrap())
		.await
		.expect("Failed to get the profile. Did you log in using the correct account?");
}

fn prompt_auth() -> Result<User, Box<dyn std::error::Error>> {
	let state = generate_state();

	let (auth_url, _, verifier_obj) =
		get_secure_login_data(constants::CLIENT_ID, constants::REDIRECT_URL, Some(state));
	open::that(auth_url.as_str()).unwrap();
	print!("Paste query string from URL (code=...&state=...):");
	io::stdout().flush().unwrap();
	let mut input = String::new();
	io::stdin().read_line(&mut input).unwrap();

	let parsed = form_urlencoded::parse(input.trim().as_bytes());

	let mut code = None;

	for (k, v) in parsed {
		match k.as_ref() {
			"code" => code = Some(v.into_owned()),
			_ => {}
		}
	}

	let rt = Runtime::new()?;
	Ok(rt.block_on(async {
		complete_login(
			constants::CLIENT_ID,
			constants::REDIRECT_URL,
			code.unwrap(),
			verifier_obj,
		)
		.await
	}))
}

pub fn login() {
	let info = prompt_auth().unwrap();
	let mut users = helpers::read_users();
	upsert_user(&mut users, info);
	if let Err(e) = helpers::write_users(users) {
		eprintln!("WARN: Failed to write: {}", e);
	}
}

pub fn logout(id: String) {
	println!("Log out {}...", id);
	// TODO: actually log out lol
}

pub fn list() {
	let users = helpers::read_users();
	println!(
		"{:<16}|{:>36}\n{}|{}",
		"Username",
		"UUID",
		"-".repeat(16),
		"-".repeat(36)
	);
	for i in users {
		println!("{:<16}|{:>36}", i.name, i.id);
	}
}
