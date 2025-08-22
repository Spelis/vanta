pub mod ms_helpers;
pub mod types;
use crate::constants;
use crate::helpers::upsert_user;
use crate::{helpers, ms_auth::ms_helpers::get_secure_login_data};
use std::io::{self, Write};
use tokio::runtime::Runtime;
use url::form_urlencoded;

fn prompt_auth() -> Result<types::User, Box<dyn std::error::Error>> {
	let state = ms_helpers::generate_state();

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
		ms_helpers::complete_login(
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
