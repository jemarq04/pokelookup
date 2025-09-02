use crate::get_name;
use crate::utils::cli::{self, LanguageId};
use clap::error::ErrorKind;
use futures::future;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::*;

pub async fn print_eggs(
  client: &RustemonClient,
  pokemon: &str,
  fast: bool,
  lang: LanguageId,
) -> Result<Vec<String>, clap::Error> {
  // Create pokemon species resource
  let species = match pokemon_species::get_by_name(&pokemon.replace(" ", "-"), &client).await {
    Ok(x) => x,
    Err(_) => {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        format!("invalid pokemon species: {pokemon}"),
      ));
    },
  };

  // Get egg group resources
  let eggs = match future::try_join_all(
    species
      .egg_groups
      .iter()
      .map(async |g| g.follow(&client).await),
  )
  .await
  {
    Ok(x) => x,
    Err(_) => {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        format!(
          "API error: could not retrieve egg groups for {}",
          species.name,
        ),
      ));
    },
  };

  // Get egg group names
  let mut egg_names = Vec::new();
  for egg in eggs.iter() {
    egg_names.push(if !fast {
      get_name!(egg, client, lang.to_string())
    } else {
      egg.name.clone()
    });
  }

  // Return egg groups
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast {
      get_name!(species, client, lang.to_string())
    } else {
      species.name.clone()
    }
  ));
  egg_names
    .iter()
    .for_each(|name| result.push(format!(" - {name}")));

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_eggs() {
    let client = RustemonClient::default();

    let success = vec![
      vec!["stantler:", " - ground"],
      vec!["Stantler:", " - Field"],
    ];

    for (idx, vals) in success.into_iter().enumerate() {
      let pokemon = String::from("stantler");
      let fast = idx == 0;
      let lang = LanguageId::En;

      match print_eggs(&client, &pokemon, fast, lang).await {
        Ok(res) => assert_eq!(res, vals),
        Err(err) => panic!("{}", err.render()),
      }
    }
  }
}
