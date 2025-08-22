use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{Rng, distr::Alphanumeric};
use reqwest::Client;
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

use crate::ms_auth::types::*;

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

/// Generates the login data for a secure login with pkce and state.
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
