use std::path::PathBuf;

use crate::helpers::{self, get_instance_folder};

mod lc_helpers;
mod lc_types;

pub fn launch(id: String, uid: String) {
	let inst_dir = get_instance_folder(&id);
	let users = helpers::read_users();
	let user = users
		.iter()
		.find(|u| uid == u.name)
		.expect("User not found.");

	let options = lc_types::Options::new(
		user.name.clone(),
		user.id.clone(),
		user.access_token.clone(),
		inst_dir.to_str().unwrap().to_string(),
	);

	let mut command: Vec<String> = vec![];

	command.push(options.default_executable_path);

	if let Some(jvm_args) = options.jvm_arguments {
		command.extend(
			jvm_args
				.iter()
				.map(|v| v.to_string())
				.collect::<Vec<String>>(),
		);
	}
	// yeah dude fuck launching the game i can't do this shit

	dbg!(command);
}
