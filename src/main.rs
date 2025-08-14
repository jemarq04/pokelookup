mod lookup;
mod utils;

use clap::Parser;
use rustemon::client::RustemonClient;
use utils::cli::{Args, SubArgs};

#[tokio::main]
async fn main() {
  let mut args = Args::parse();

  // Create cache directory for API calls
  if let None = args.cache_dir {
    args.cache_dir = match std::env::home_dir() {
      Some(path) => Some(format!("{}/.cache/pokelookup", path.display()).into()),
      None => None,
    }
  }
  let client = match args.cache_dir {
    Some(path) => {
      match rustemon::client::RustemonClientBuilder::default()
        .with_manager(rustemon::client::CACacheManager::new(path, false))
        .try_build()
      {
        Ok(cl) => cl,
        Err(_) => {
          eprintln!("warning: cache directory set to cache manager default");
          RustemonClient::default()
        },
      }
    },
    None => {
      eprintln!("warning: cache directory set to cache manager default");
      RustemonClient::default()
    },
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
    SubArgs::DexCmd {
      endpoint,
      generation,
      ..
    } => {
      if let Some(x) = endpoint.pokemon {
        lookup::dex::open_pokedex(x, generation)
      } else if let Some(_x) = endpoint.region {
        lookup::dex::open_pokearth()
      } else if let Some(_x) = endpoint.move_ {
        lookup::dex::open_attackdex()
      } else if let Some(_x) = endpoint.ability {
        lookup::dex::open_abilitydex()
      } else if let Some(_x) = endpoint.item {
        lookup::dex::open_itemdex()
      } else {
        unreachable!()
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
}
