mod args;
mod constants;
mod download;
mod helpers;
mod launch;
mod ms_auth;
use clap::Parser;

fn main() {
	let cli: args::Cli = args::Cli::parse();
	match cli.sub {
		args::SubCmd::User(user_cmd) => match user_cmd.cmd {
			args::UserSub::Login => ms_auth::login(),
			args::UserSub::List => ms_auth::list(),
			args::UserSub::Logout { id } => ms_auth::logout(id),
		},
		args::SubCmd::Instance(inst_cmd) => match inst_cmd.cmd {
			args::InstanceSub::VersionList => {
				download::list_versions().expect("Failed to list versions")
			}
			args::InstanceSub::List => println!("List"),
			args::InstanceSub::Run { id, uid } => launch::launch(id, uid),
			args::InstanceSub::New {
				id,
				version,
				parallel,
			} => download::new_instance(version, id, parallel),
		},
		args::SubCmd::Modloader(modldr_cmd) => match modldr_cmd.cmd {
			args::LoaderSub::Install { loader } => println!("Install {}", loader),
		},
	}
}
