mod lookup;
mod utils;

use clap::Parser;
use rustemon::client::RustemonClient;
use std::{fs::create_dir, path::Path};
use utils::cli::{Args, SubArgs, get_appname};

#[cfg(feature = "web")]
use clap::error::ErrorKind;
#[cfg(feature = "web")]
use utils::{cli, cli::DexMode};

#[tokio::main]
async fn main() {
  let mut args = Args::parse();

  // Create cache directory for API calls
  args.cache_dir = match args.cache_dir {
    Some(p) => Some(p),
    None => {
      let mut result = None;
      if let Some(home) = std::env::home_dir() {
        let dirpath = format!("{}/.cache", home.display());
        let dirpath = Path::new(&dirpath);
        if dirpath.exists() || matches!(create_dir(dirpath), Ok(_)) {
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
    SubArgs::ListCmd {
      pokemon,
      fast,
      lang,
    } => lookup::print_varieties(&client, &pokemon, fast, lang).await,
    SubArgs::TypeCmd {
      pokemon,
      fast,
      lang,
      recursive,
    } => lookup::print_types(&client, &pokemon, fast, lang, recursive).await,
    SubArgs::AbilityCmd {
      pokemon,
      fast,
      lang,
      recursive,
    } => lookup::print_abilities(&client, &pokemon, fast, lang, recursive).await,
    SubArgs::MoveCmd {
      pokemon,
      fast,
      lang,
      vgroup,
      level,
    } => lookup::print_moves(&client, &pokemon, fast, lang, vgroup, level).await,
    SubArgs::EggCmd {
      pokemon,
      fast,
      lang,
    } => lookup::print_eggs(&client, &pokemon, fast, lang).await,
    SubArgs::GenderCmd {
      pokemon,
      fast,
      lang,
    } => lookup::print_genders(&client, &pokemon, fast, lang).await,
    SubArgs::EncounterCmd {
      version,
      pokemon,
      fast,
      lang,
      recursive,
    } => lookup::print_encounters(&client, version, &pokemon, fast, lang, recursive).await,
    SubArgs::EvolutionCmd {
      pokemon,
      fast,
      lang,
      secret,
      all,
    } => lookup::print_evolutions(&client, &pokemon, fast, lang, secret, all).await,
    SubArgs::MatchupCmd {
      primary,
      secondary,
      list,
      fast,
      lang,
    } => lookup::print_matchups(&client, primary, secondary, list, fast, lang).await,
    #[cfg(feature = "web")]
    SubArgs::SearchCmd {
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
    Ok(s) if s.len() == 0 => println!("No results found."),
    Ok(s) => s.iter().for_each(|x| println!("{}", x)),
    Err(err) => err.exit(),
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cli_varieties() {
    let args = svec![get_appname(), "list", "-f", "meowth"];
    match Args::try_parse_from(args) {
      Err(e) => panic!("{}", e.render()),
      Ok(_) => {},
    }
  }
}
