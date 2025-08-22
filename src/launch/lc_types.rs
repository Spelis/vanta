use serde::{Deserialize, Serialize};
use serde_json;

use crate::helpers::get_instance_folder;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
	pub username: String,
	pub uuid: String,
	pub token: String,
	pub executable_path: String,
	pub default_executable_path: String,
	pub jvm_arguments: Option<Vec<serde_json::Value>>,
	pub launcher_name: String,
	pub launcher_version: String,
	pub game_directory: String,
	pub demo: bool,
	pub custom_resolution: bool,
	pub resolution_width: String,
	pub resolution_height: String,
	pub server: String,
	pub port: String,
	pub natives_directory: String,
	pub enable_logging_config: bool,
	pub disable_multiplayer: bool,
	pub disable_chat: bool,
	pub quick_play_path: Option<serde_json::Value>,
	pub quick_play_singleplayer: Option<serde_json::Value>,
	pub quick_play_multiplayer: Option<serde_json::Value>,
	pub quick_play_realms: Option<serde_json::Value>,
}

impl Options {
	pub fn new(username: String, uuid: String, token: String, inst_id: String) -> Self {
		Self {
			username,
			uuid,
			token,
			executable_path: "java".to_string(),
			default_executable_path: "java".to_string(),
			jvm_arguments: Some(Vec::new()),
			launcher_name: "vanta-launcher".to_string(),
			launcher_version: "1.0".to_string(),
			game_directory: get_instance_folder(&inst_id).to_str().unwrap().to_string(),
			demo: false,
			custom_resolution: false,
			resolution_width: "800".to_string(),
			resolution_height: "600".to_string(),
			server: "".to_string(),
			port: "25565".to_string(),
			natives_directory: get_instance_folder(&inst_id)
				.join("versions/natives/")
				.to_str()
				.unwrap()
				.to_string(),
			enable_logging_config: false,
			disable_multiplayer: false,
			disable_chat: false,
			quick_play_path: None,
			quick_play_singleplayer: None,
			quick_play_multiplayer: None,
			quick_play_realms: None,
		}
	}
}
