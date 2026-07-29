use crate::get_name;
use crate::utils::args::TypeArgs;
use crate::utils::cli;
use crate::utils::helpers;
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;

pub async fn print_types(
  client: &RustemonClient,
  args: TypeArgs,
) -> Result<Vec<String>, clap::Error> {
  // Create pokemon resources
  let resources = match helpers::get_pokemon_from_chain(client, &args.pokemon, args.recursive).await
  {
    Ok(x) => x,
    Err(_) => {
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
    },
  };

  // Iterate over all requested pokemon
  let mut result = Vec::new();
  for mon_resource in resources.iter() {
    // Get type names
    let mut type_names = Vec::new();
    for item in mon_resource.types.iter() {
      type_names.push(if !args.fast {
        get_name!(follow item.type_, client, args.lang.to_string())
      } else {
        item.type_.name.clone()
      });
    }
    if let Some(generation) = args.generation {
      for past_item in mon_resource.past_types.iter() {
        if let Ok(x) = past_item.generation.follow(client).await
          && x.id >= generation
        {
          type_names.clear();
          for item in past_item.types.iter() {
            type_names.push(if !args.fast {
              get_name!(follow item.type_, client, args.lang.to_string())
            } else {
              item.type_.name.clone()
            });
          }
        }
      }
    }

    // Return types
    result.push(format!(
      "{}:",
      if !args.fast {
        helpers::get_pokemon_name(client, mon_resource, &args.lang.to_string()).await
      } else {
        mon_resource.name.clone()
      }
    ));
    result.push(format!("  {}", type_names.join("/")));
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::enums::LanguageId;

  #[tokio::test]
  async fn test_types() {
    let client = RustemonClient::default();

    let success: Vec<String> = vec!["Toxel:", "  Electric/Poison"]
      .into_iter()
      .map(|x| x.into())
      .collect();

    for fast in [false, true].into_iter() {
      let args = TypeArgs {
        pokemon: String::from("toxel"),
        fast,
        generation: None,
        lang: LanguageId::En,
        recursive: false,
      };

      match print_types(&client, args).await {
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
  async fn test_past_types() {
    let client = RustemonClient::default();

    let success: Vec<String> = vec!["jigglypuff:", "  normal"]
      .into_iter()
      .map(|x| x.into())
      .collect();

    for generation in 1..=5 {
      let args = TypeArgs {
        pokemon: String::from("jigglypuff"),
        fast: true,
        generation: Some(generation),
        lang: LanguageId::En,
        recursive: false,
      };

      match print_types(&client, args).await {
        Ok(s) => assert_eq!(
          s,
          success
            .iter()
            .map(|x| x.to_lowercase())
            .collect::<Vec<String>>()
        ),
        Err(err) => panic!("{}", err.render()),
      }
    }
  }

  #[tokio::test]
  async fn test_types_recursive() {
    let client = RustemonClient::default();

    let success = vec!["stantler:", "  normal", "wyrdeer:", "  normal/psychic"];
    let args = TypeArgs {
      pokemon: String::from("stantler"),
      fast: true,
      generation: None,
      lang: LanguageId::En,
      recursive: true,
    };

    match print_types(&client, args).await {
      Ok(s) => assert_eq!(s, success),
      Err(err) => panic!("{}", err.render()),
    }
  }
}
