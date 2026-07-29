mod lookup;
mod utils;

use clap::Parser;
use rustemon::client::RustemonClient;
use std::{fs::create_dir, path::Path};
use utils::cli::{Cli, Command, get_appname};

#[cfg(feature = "web")]
use clap::error::ErrorKind;
#[cfg(feature = "web")]
use utils::{cli, cli::DexMode};

#[tokio::main]
async fn main() {
  let mut args = Cli::parse();

  // Create cache directory for API calls
  args.cache_dir = match args.cache_dir {
    Some(p) => Some(p),
    None => {
      let mut result = None;
      if let Some(home) = std::env::home_dir() {
        let dirpath = format!("{}/.cache", home.display());
        let dirpath = Path::new(&dirpath);
        if dirpath.exists() || create_dir(dirpath).is_ok() {
          result = Some(format!("{}/{}", dirpath.display(), get_appname()).into());
        }
      }
      result
    },
  };
  let client = if let Some(path) = args.cache_dir
    && let Ok(client) = rustemon::client::RustemonClientBuilder::default()
      .with_manager(rustemon::client::CACacheManager::new(path, false))
      .try_build()
  {
    client
  } else {
    eprintln!("warning: cache directory set to cache manager default");
    RustemonClient::default()
  };

  // Call the appropriate subcommand for results
  let result = match args.command {
    Command::ListCmd(args) => lookup::print_varieties(&client, args).await,
    Command::TypeCmd(args) => lookup::print_types(&client, args).await,
    Command::AbilityCmd(args) => lookup::print_abilities(&client, args).await,
    Command::MoveCmd(args) => lookup::print_moves(&client, args).await,
    Command::EggCmd(args) => lookup::print_eggs(&client, args).await,
    Command::GenderCmd(args) => lookup::print_genders(&client, args).await,
    Command::EncounterCmd(args) => lookup::print_encounters(&client, args).await,
    Command::EvolutionCmd(args) => lookup::print_evolutions(&client, args).await,
    Command::MatchupCmd(args) => lookup::print_matchups(&client, args).await,
    #[cfg(feature = "web")]
    Command::WebCmd(args) => {
      let url = match args.endpoint.get_mode() {
        DexMode::Pokedex(name) => lookup::dex::open_pokedex(name, args.generation),
        DexMode::Pokearth(name) => lookup::dex::open_pokearth(name, args.area, args.generation),
        DexMode::Attackdex(name) => lookup::dex::open_attackdex(name, args.generation),
        DexMode::Abilitydex(name) => lookup::dex::open_abilitydex(name),
        DexMode::Itemdex(name) => lookup::dex::open_itemdex(name),
      };
      match url {
        Ok(url) => match open::that(&url) {
          Ok(_) => {
            if args.quiet {
              return;
            }
            Ok(svec!["Opened page successfully."])
          },
          Err(_) => Err(cli::error(
            ErrorKind::InvalidValue,
            format!("couldn't open URL: {url}"),
          )),
        },
        Err(e) => Err(e),
      }
    },
  };

  // Handle output
  match result {
    Ok(s) if s.is_empty() => println!("No results found."),
    Ok(s) => s.iter().for_each(|x| println!("{x}")),
    Err(err) => err.exit(),
  };
}

#[cfg(test)]
mod tests {}
