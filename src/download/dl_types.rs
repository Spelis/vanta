use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::download::dl_helpers;

#[derive(Debug, Clone)]
pub struct DownloadEntry {
	pub url: String,
	pub destination: String,
	pub size: Option<usize>,
	pub sha1: Option<String>,
	pub name: Option<String>,
	pub executable: bool,
}

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
	pub latest: VersionManifestLatest,
	pub versions: Vec<VersionManifestVersion>,
}

#[derive(Debug, Deserialize)]
pub struct VersionManifestLatest {
	pub release: String,
	pub snapshot: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionManifestVersion {
	pub id: String,
	pub r#type: String,
	pub url: String,
	pub time: String,
	#[allow(non_snake_case)]
	pub releaseTime: String,
	pub sha1: String,
	#[allow(non_snake_case)]
	pub complianceLevel: i8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionJson {
	pub arguments: serde_json::Value,
	#[allow(non_snake_case)]
	pub assetIndex: serde_json::Value,
	pub downloads: serde_json::Value,
	pub libraries: Vec<serde_json::Value>,
	#[serde(flatten)]
	pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftAsset {
	pub hash: String,
	pub size: usize,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftAssets {
	pub objects: HashMap<String, MinecraftAsset>,
}
