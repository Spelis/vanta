use std::fs;
use std::path::PathBuf;
use std::thread;

use tokio::runtime::Runtime;

use crate::helpers;
use crate::helpers::get_instance_folder;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use std::vec;

use reqwest::Client;

pub fn download(
	entry: &mut DownloadEntry,
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
	version: &VersionJson,
) -> Result<Vec<DownloadEntry>, Box<dyn std::error::Error>> {
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
	version: &VersionJson,
) -> Result<Vec<DownloadEntry>, Box<dyn std::error::Error>> {
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

pub fn new_instance(version: String, id: String, dl_threads: usize) {
	let folder = get_instance_folder(&version);
	_ = fs::create_dir_all(folder);
	_ = install_minecraft(version, id, dl_threads);
}

pub fn install_minecraft(
	version: String,
	instance: String,
	threads: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	let rt = Runtime::new()?;
	let versions = rt.block_on(async { get_version_manifest().await }).unwrap();

	let valid_version = match versions.versions.iter().find(|v| v.id == version) {
		Some(v) => v,
		None => panic!("Invalid Version"),
	};

	let version_json = rt
		.block_on(async { get_version_json(valid_version.url.clone()).await })
		.unwrap();

	let mut queue: Vec<DownloadEntry> = vec![];

	let asset_queue = rt
		.block_on(async { queue_assets(&version_json).await })
		.unwrap();

	let lib_queue = rt
		.block_on(async { queue_libs(&version_json).await })
		.unwrap();

	queue.extend(asset_queue);
	queue.extend(lib_queue);
	queue.push(DownloadEntry {
		url: version_json.downloads["client"]["url"]
			.as_str()
			.unwrap()
			.to_string(),
		destination: "versions/client.jar".to_string(),
		size: Some(version_json.downloads["client"]["size"].as_u64().unwrap() as usize),
		sha1: Some("".to_string()),
		name: Some("Client".to_string()),
		executable: true,
	});

	_ = helpers::write_bytes(
		PathBuf::from(get_instance_folder(&instance).clone())
			.join("versions/client.json")
			.to_str()
			.unwrap()
			.to_string(),
		serde_json::to_string(&version_json).unwrap().as_bytes(),
	);

	let qlen = queue.len();
	let chunk_size = (qlen + threads - 1) / threads;

	let handles: Vec<_> = (0..threads)
		.map(|i| {
			let mut queue_chunk: Vec<_> =
				queue[i * chunk_size..std::cmp::min((i + 1) * chunk_size, qlen)].to_vec(); // clones entries

			let inst_id = instance.clone();
			thread::spawn(move || {
				for mut e in queue_chunk.iter_mut() {
					println!(
						"Downloading {} ({} bytes)",
						e.name.clone().map_or("_".to_string(), |v| v),
						e.size.map_or(0, |v| v)
					);

					if let Err(err) = download(&mut e, get_instance_folder(&inst_id)) {
						eprintln!(
							"Failed to download {}: {}",
							e.name.clone().unwrap_or("_".to_string()),
							err
						);
					}
				}
			})
		})
		.collect();

	for handle in handles {
		_ = handle.join();
	}

	Ok(())
}

pub fn list_versions() -> Result<(), Box<dyn std::error::Error>> {
	let rt = Runtime::new()?;
	let mut versions = rt.block_on(async { get_version_manifest().await }).unwrap();

	versions.versions.reverse();

	println!(
		"{:<20}|{:^11}|{:>11}\n{}|{}|{}",
		"ID",
		"Type",
		"Released",
		"-".repeat(20),
		"-".repeat(11),
		"-".repeat(11)
	); // longest version id is 22w13oneblockatatime (several versions equal length)
	for version in versions.versions {
		println!(
			"{:<20}|{:^11}|{:>11}",
			version.id,
			version.r#type,
			&version.releaseTime[0..10]
		);
	}
	Ok(())
}
