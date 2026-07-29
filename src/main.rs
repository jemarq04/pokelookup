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
    Command::AbilityCmd {
      pokemon,
      fast,
      lang,
      recursive,
    } => lookup::print_abilities(&client, &pokemon, fast, lang, recursive).await,
    Command::MoveCmd {
      pokemon,
      fast,
      lang,
      vgroup,
      level,
    } => lookup::print_moves(&client, &pokemon, fast, lang, vgroup, level).await,
    Command::EggCmd {
      pokemon,
      fast,
      lang,
    } => lookup::print_eggs(&client, &pokemon, fast, lang).await,
    Command::GenderCmd {
      pokemon,
      fast,
      lang,
    } => lookup::print_genders(&client, &pokemon, fast, lang).await,
    Command::EncounterCmd {
      version,
      pokemon,
      fast,
      lang,
      recursive,
      condensed,
    } => {
      lookup::print_encounters(&client, version, &pokemon, fast, lang, recursive, condensed).await
    },
    Command::EvolutionCmd {
      pokemon,
      fast,
      lang,
      secret,
      all,
    } => lookup::print_evolutions(&client, &pokemon, fast, lang, secret, all).await,
    Command::MatchupCmd {
      primary,
      secondary,
      list,
      fast,
      lang,
    } => lookup::print_matchups(&client, primary, secondary, list, fast, lang).await,
    #[cfg(feature = "web")]
    Command::WebCmd {
      endpoint,
      generation,
      area,
      quiet,
    } => {
      let url = match endpoint.get_mode() {
        DexMode::Pokedex(name) => lookup::dex::open_pokedex(name, generation),
        DexMode::Pokearth(name) => lookup::dex::open_pokearth(name, area, generation),
        DexMode::Attackdex(name) => lookup::dex::open_attackdex(name, generation),
        DexMode::Abilitydex(name) => lookup::dex::open_abilitydex(name),
        DexMode::Itemdex(name) => lookup::dex::open_itemdex(name),
      };
      match url {
        Ok(url) => match open::that(&url) {
          Ok(_) => {
            if quiet {
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
