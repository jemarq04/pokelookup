use crate::get_name;
use crate::utils::args::AbilityArgs;
use crate::utils::cli;
use crate::utils::helpers;
use clap::error::ErrorKind;
use futures::future;
use rustemon::Follow;
use rustemon::client::RustemonClient;

pub async fn print_abilities(
  client: &RustemonClient,
  args: AbilityArgs,
) -> Result<Vec<String>, clap::Error> {
  // Create struct to store ability
  struct Ability {
    hidden: bool,
    ability: rustemon::model::pokemon::Ability,
  }

  // Create pokemon resources
  let Ok(resources) = helpers::get_pokemon_from_chain(client, &args.pokemon, args.recursive).await
  else {
    let valid = cli::VALID;
    let err = cli::error(
      ErrorKind::InvalidValue,
      format!(
        "invalid pokemon: {1}\n\n{valid}tip:{valid:#} try running '{} list {}'",
        cli::get_appname(),
        args.pokemon,
      ),
    );
    return Err(err);
  };

  // Iterate over all requested pokemon
  let mut result = Vec::new();
  for mon_resource in &resources {
    // Get ability resources
    let Ok(abilities) = future::try_join_all(mon_resource.abilities.iter().map(async |a| {
      match a.ability.clone().unwrap().follow(client).await {
        Ok(x) => Ok(Ability {
          hidden: a.is_hidden,
          ability: x,
        }),
        Err(_) => Err(()),
      }
    }))
    .await
    else {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        format!(
          "API error: could not retrieve abilities for {}",
          mon_resource.name,
        ),
      ));
    };

    // Get ability names
    let mut names = Vec::new();
    for ab in abilities {
      names.push(if args.fast {
        ab.ability.name.clone() + if ab.hidden { " (hidden)" } else { "" }
      } else {
        get_name!(ab.ability, client, args.lang.to_string())
          + if ab.hidden { " (Hidden)" } else { "" }
      });
    }

    // Return abilities
    result.push(format!(
      "{}:",
      if args.fast {
        mon_resource.name.clone()
      } else {
        helpers::get_pokemon_name(client, mon_resource, &args.lang.to_string()).await
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
  use crate::utils::enums::LanguageId;

  #[tokio::test]
  async fn test_abilities() {
    let client = RustemonClient::default();

    let success: Vec<String> = vec!["Toxel:", " 1. Rattled", " 2. Static", " 3. Klutz (Hidden)"]
      .into_iter()
      .map(std::convert::Into::into)
      .collect();

    for fast in [false, true] {
      let args = AbilityArgs {
        pokemon: String::from("toxel"),
        fast,
        lang: LanguageId::En,
        recursive: false,
      };

      match print_abilities(&client, args).await {
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

    let args = AbilityArgs {
      pokemon: String::from("stantler"),
      fast: false,
      lang: LanguageId::En,
      recursive: true,
    };

    match print_abilities(&client, args).await {
      Ok(s) => assert_eq!(s, success),
      Err(err) => panic!("{}", err.render()),
    }
  }
}
