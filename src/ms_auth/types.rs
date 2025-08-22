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
