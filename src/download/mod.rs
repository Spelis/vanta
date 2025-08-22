use std::fs;
use std::path::PathBuf;
use std::thread;

use tokio::runtime::Runtime;

use crate::download::dl_helpers::queue_libs;
use crate::helpers;
use crate::{
	download::{dl_helpers::queue_assets, dl_types::DownloadEntry},
	helpers::get_instance_folder,
};

mod dl_helpers;
mod dl_types;

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
	let versions = rt
		.block_on(async { dl_helpers::get_version_manifest().await })
		.unwrap();

	let valid_version = match versions.versions.iter().find(|v| v.id == version) {
		Some(v) => v,
		None => panic!("Invalid Version"),
	};

	let version_json = rt
		.block_on(async { dl_helpers::get_version_json(valid_version.url.clone()).await })
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

					if let Err(err) = dl_helpers::download(&mut e, get_instance_folder(&inst_id)) {
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
	let mut versions = rt
		.block_on(async { dl_helpers::get_version_manifest().await })
		.unwrap();

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
