use std::{
	fs::{self, File},
	io::{BufReader, Write},
	path::{Path, PathBuf},
};

use crate::ms_auth::User;

pub const USER_FILE: &str = "accounts.json";

/// Write to the account file.
pub fn write_users(users: Vec<User>) -> Result<(), std::io::Error> {
	let json = serde_json::to_string_pretty(&users)?;
	let mut file = File::create(get_data_folder(Some(USER_FILE)))?;
	file.write_all(json.as_bytes())?;
	file.sync_all()?;
	Ok(())
}

pub fn write_bytes(filename: String, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
	match PathBuf::from(&filename).parent() {
		Some(p) => fs::create_dir_all(p).expect("Failed to create directory."),
		None => (),
	};
	let mut file = File::create(&filename)?;
	file.write_all(data)?;
	file.sync_all()?;
	Ok(())
}

/// Read the account file.
pub fn read_users() -> Vec<User> {
	match File::open(get_data_folder(Some(USER_FILE))) {
		Ok(file) => {
			let reader = BufReader::new(file);
			match serde_json::from_reader(reader) {
				Ok(users) => users,
				Err(_) => {
					vec![]
				}
			}
		}
		Err(_) => {
			vec![]
		}
	}
}

/// Insert or update user depending on if it exists or not
pub fn upsert_user(vec: &mut Vec<User>, new_user: User) {
	if let Some(existing) = vec.iter_mut().find(|u| u.name == new_user.name) {
		*existing = new_user;
	} else {
		vec.push(new_user);
	}
}

pub fn get_data_folder(append: Option<&str>) -> PathBuf {
	let path = platform_dirs::AppDirs::new(Some("vanta"), true)
		.unwrap()
		.data_dir;

	if let Some(append_str) = append {
		path.join(Path::new(append_str))
	} else {
		path
	}
}

pub fn get_instance_folder(instance: &str) -> PathBuf {
	get_data_folder(Some(&format!("instances/{}", instance)))
}
