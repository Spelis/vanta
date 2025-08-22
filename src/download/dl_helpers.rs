use std::{path::PathBuf, vec};

use reqwest::Client;

use crate::{
	download::dl_types::{self, DownloadEntry, MinecraftAssets, VersionJson, VersionManifest},
	helpers,
};

pub fn download(
	entry: &mut dl_types::DownloadEntry,
	prefix: PathBuf,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
	let resp = reqwest::blocking::get(entry.url.clone())?;
	let bytes = &resp.bytes()?;
	helpers::write_bytes(
		prefix
			.join(entry.destination.clone())
			.to_string_lossy()
			.to_string(),
		&bytes.clone(),
	)?;

	Ok(bytes.to_vec())
}

pub async fn get_version_manifest() -> Result<VersionManifest, Box<dyn std::error::Error>> {
	let client = Client::new();
	let raw_resp = client
		.get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;

	let resp = serde_json::from_str::<VersionManifest>(&raw_resp).unwrap();

	Ok(resp)
}

pub async fn get_version_json(url: String) -> Result<VersionJson, Box<dyn std::error::Error>> {
	let client = Client::new();
	let raw_resp = client
		.get(&url)
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;

	let resp = serde_json::from_str::<VersionJson>(&raw_resp).unwrap();

	Ok(resp)
}

pub async fn get_assets_vec(url: String) -> Result<MinecraftAssets, Box<dyn std::error::Error>> {
	let client = Client::new();
	let raw_resp = client
		.get(&url)
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;

	let resp = serde_json::from_str::<MinecraftAssets>(&raw_resp).unwrap();

	Ok(resp)
}

pub async fn queue_assets(
	version: &dl_types::VersionJson,
) -> Result<Vec<dl_types::DownloadEntry>, Box<dyn std::error::Error>> {
	let assets: MinecraftAssets =
		get_assets_vec(version.assetIndex["url"].as_str().unwrap().to_string()).await?;
	let mut queue: Vec<DownloadEntry> = vec![];

	for (k, a) in assets.objects.iter() {
		queue.push(DownloadEntry {
			url: format!(
				"https://resources.download.minecraft.net/{}/{}",
				&a.hash[0..2],
				a.hash
			),
			destination: format!("assets/objects/{}/{}", &a.hash[0..2], a.hash),
			size: Some(a.size),
			sha1: None,
			name: Some(k.to_string()),
			executable: false,
		});
	}

	Ok(queue)
}

pub async fn queue_libs(
	version: &dl_types::VersionJson,
) -> Result<Vec<dl_types::DownloadEntry>, Box<dyn std::error::Error>> {
	let mut queue: Vec<DownloadEntry> = vec![];

	for l in version.libraries.iter() {
		// TODO: check for OS to make sure we aren't downloading
		// anything we dont need
		let dl = l["downloads"]["artifact"].clone();
		queue.push(DownloadEntry {
			size: Some(dl["size"].as_u64().unwrap() as usize),
			destination: format!("libraries/{}", &dl["path"].as_str().unwrap()),
			name: Some(dl["path"].as_str().unwrap().to_string()),
			url: dl["url"].as_str().unwrap().to_string(),
			sha1: Some(dl["sha1"].as_str().unwrap().to_string()),
			executable: false,
		})
	}

	Ok(queue)
}
