use crate::utils::args;
use clap::builder::styling::{AnsiColor, Effects, Style, Styles};
use clap::{CommandFactory, Parser, Subcommand};

pub const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
pub const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
pub const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

// Cargo's color style
// [source](https://github.com/crate-ci/clap-cargo/blob/master/src/style.rs)
const CARGO_STYLING: Styles = Styles::styled()
  .header(HEADER)
  .usage(USAGE)
  .literal(LITERAL)
  .placeholder(PLACEHOLDER)
  .error(ERROR)
  .valid(VALID)
  .invalid(INVALID);

// ========================================================================================================================
// CLI Parser and Subcommands
// ========================================================================================================================

/// Look up pokemon details using PokeAPI using the 'rustemon' wrapper. Note that sometimes pokemon need to be listed
/// with their forms if the form is distinct enough (e.g. pumkaboo-small or toxtricity-amped). These varieties can be
/// listed using the 'list' subcommand.
#[derive(Parser, Debug)]
#[command(version, long_about, styles=CARGO_STYLING)]
pub struct Cli {
  #[arg(
    long,
    value_name = "DIR",
    help = "cache directory for API calls (default: ~/.cache/pokelookup/)"
  )]
  pub cache_dir: Option<std::path::PathBuf>,

  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
  /// Look up the varieties of a given pokemon.
  #[command(name = "list", long_about)]
  ListCmd(args::ListArgs),

  /// Look up the type(s) of a given pokemon.
  #[command(name = "types", long_about)]
  TypeCmd(args::TypeArgs),

  /// Look up the abilities of a given pokemon. If the ability is a hidden ability, it will be
  /// marked accordingly.
  #[command(
    name = "abilities",
    about = "Look up the abilities of a given pokemon",
    long_about
  )]
  AbilityCmd(args::AbilityArgs),

  /// Look up the level-up moveset of a given pokemon. If a level is provided
  /// then the four moves at or below the given level are provided. By default, this will
  /// retrieve the moveset from the Scarlet/Violet version group.
  #[command(
    name = "moves",
    about = "Look up the level-up moveset of a given pokemon",
    long_about
  )]
  MoveCmd(args::MoveArgs),

  /// Look up the egg groups of a given pokemon species.
  #[command(name = "eggs", long_about)]
  EggCmd(args::EggArgs),

  /// Look up the gender ratio of a given pokemon species.
  #[command(name = "genders", long_about)]
  GenderCmd(args::GenderArgs),

  /// Look up the encounters for a given pokemon and version.
  #[command(name = "encounters", long_about)]
  EncounterCmd(args::EncounterArgs),

  /// Look up evolution chain for a given pokemon species.
  #[command(name = "evolutions", long_about)]
  EvolutionCmd(args::EvolutionArgs),

  /// Look up the type weaknesses/resistances for given type(s).
  #[command(name = "matchups", long_about)]
  MatchupCmd(args::MatchupArgs),

  /// Open web pages for a given endpoint. A valid endpoint includes pokemon, abilities, items, and more.
  #[cfg(feature = "web")]
  #[command(name = "web", long_about)]
  WebCmd(args::WebArgs),
}

// ========================================================================================================================
// Helper Functions
// ========================================================================================================================

pub fn get_appname() -> String {
  String::from(Cli::command().get_name())
}

pub fn error(kind: clap::error::ErrorKind, message: String) -> clap::Error {
  Cli::command().error(kind, message)
}
