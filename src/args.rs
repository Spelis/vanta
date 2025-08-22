use clap::{
	Parser, Subcommand,
	builder::styling::{AnsiColor, Color, Style},
};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None,styles=get_styles())]
pub struct Cli {
	#[command(subcommand)]
	pub sub: SubCmd,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SubCmd {
	/// User-related commands
	User(UserCmd),
	/// Instance-related commands
	Instance(InstanceCmd),
	/// Modloader command
	Modloader(LoaderCmd),
}

#[derive(Parser, Debug, Clone)]
pub struct UserCmd {
	#[command(subcommand)]
	pub cmd: UserSub,
}

#[derive(Subcommand, Debug, Clone)]
pub enum UserSub {
	Login,
	Logout { id: String },
	List,
}

#[derive(Parser, Debug, Clone)]
pub struct InstanceCmd {
	#[command(subcommand)]
	pub cmd: InstanceSub,
}

#[derive(Subcommand, Debug, Clone)]
pub enum InstanceSub {
	Run {
		id: String,
		uid: String,
	},
	New {
		id: String,
		version: String,
		#[arg(long, default_value_t = 4)]
		parallel: usize,
	},
	List,
	VersionList,
}

#[derive(Parser, Debug, Clone)]
pub struct LoaderCmd {
	#[command(subcommand)]
	pub cmd: LoaderSub,
}

#[derive(Subcommand, Debug, Clone)]
pub enum LoaderSub {
	Install { loader: String },
}

pub fn get_styles() -> clap::builder::Styles {
	clap::builder::Styles::styled()
		.usage(
			Style::new()
				.bold()
				.underline()
				.fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
		)
		.header(
			Style::new()
				.bold()
				.underline()
				.fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
		)
		.literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
		.invalid(
			Style::new()
				.bold()
				.fg_color(Some(Color::Ansi(AnsiColor::Red))),
		)
		.error(
			Style::new()
				.bold()
				.fg_color(Some(Color::Ansi(AnsiColor::Red))),
		)
		.valid(
			Style::new()
				.bold()
				.underline()
				.fg_color(Some(Color::Ansi(AnsiColor::Green))),
		)
		.placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))))
}
