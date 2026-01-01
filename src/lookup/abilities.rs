use crate::get_name;
use crate::utils::cli;
use crate::utils::enums::LanguageId;
use crate::utils::helpers;
use clap::error::ErrorKind;
use futures::future;
use rustemon::Follow;
use rustemon::client::RustemonClient;

pub async fn print_abilities(
  client: &RustemonClient,
  pokemon: &str,
  fast: bool,
  lang: LanguageId,
  recursive: bool,
) -> Result<Vec<String>, clap::Error> {
  // Create pokemon resources
  let resources = match helpers::get_pokemon_from_chain(&client, &pokemon, recursive).await {
    Ok(x) => x,
    Err(_) => {
      let valid = cli::VALID;
      let err = cli::error(
        ErrorKind::InvalidValue,
        format!(
          "invalid pokemon: {pokemon}\n\n{valid}tip:{valid:#} try running '{} list {pokemon}'",
          cli::get_appname()
        ),
      );
      return Err(err);
    },
  };

  // Create struct to store ability
  struct Ability {
    hidden: bool,
    ability: rustemon::model::pokemon::Ability,
  }

  // Iterate over all requested pokemon
  let mut result = Vec::new();
  for mon_resource in resources.iter() {
    // Get ability resources
    let abilities = match future::try_join_all(mon_resource.abilities.iter().map(async |a| {
      match a.ability.follow(&client).await {
        Ok(x) => Ok(Ability {
          hidden: a.is_hidden,
          ability: x,
        }),
        Err(_) => Err(()),
      }
    }))
    .await
    {
      Ok(x) => x,
      Err(_) => {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!(
            "API error: could not retrieve abilities for {}",
            mon_resource.name,
          ),
        ));
      },
    };

    // Get ability names
    let mut names = Vec::new();
    for ab in abilities.into_iter() {
      names.push(if !fast {
        get_name!(ab.ability, client, lang.to_string()) + if ab.hidden { " (Hidden)" } else { "" }
      } else {
        ab.ability.name.clone() + if ab.hidden { " (hidden)" } else { "" }
      });
    }

    // Return abilities
    result.push(format!(
      "{}:",
      if !fast {
        helpers::get_pokemon_name(&client, &mon_resource, &lang.to_string()).await
      } else {
        mon_resource.name.clone()
      }
    ));
    names
      .iter()
      .enumerate()
      .for_each(|x| result.push(format!(" {}. {}", x.0 + 1, x.1)));
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_abilities() {
    let client = RustemonClient::default();

    let success: Vec<String> = vec!["Toxel:", " 1. Rattled", " 2. Static", " 3. Klutz (Hidden)"]
      .into_iter()
      .map(|x| x.into())
      .collect();

    for fast in [false, true].into_iter() {
      let pokemon = String::from("toxel");
      let lang = LanguageId::En;
      let recursive = false;

      match print_abilities(&client, &pokemon, fast, lang, recursive).await {
        Ok(s) => assert_eq!(
          s,
          if fast {
            success.iter().map(|x| x.to_lowercase()).collect()
          } else {
            success.clone()
          }
        ),
        Err(err) => panic!("{}", err.render()),
      }
    }
  }

  #[tokio::test]
  async fn test_abilities_recursive() {
    let client = RustemonClient::default();

    let success = vec![
      "Stantler:", " 1. Intimidate", " 2. Frisk", " 3. Sap Sipper (Hidden)", "Wyrdeer:",
      " 1. Intimidate", " 2. Frisk", " 3. Sap Sipper (Hidden)",
    ];

    let pokemon = String::from("stantler");
    let fast = false;
    let lang = LanguageId::En;
    let recursive = true;

    match print_abilities(&client, &pokemon, fast, lang, recursive).await {
      Ok(s) => assert_eq!(s, success),
      Err(err) => panic!("{}", err.render()),
    }
  }
}
