use crate::get_name;
use crate::utils::cli;
use crate::utils::enums::LanguageId;
use crate::utils::helpers;
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;

pub async fn print_types(
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

  // Iterate over all requested pokemon
  let mut result = Vec::new();
  for mon_resource in resources.iter() {
    // Get type names
    let mut type_names = Vec::new();
    for item in mon_resource.types.iter() {
      type_names.push(if !fast {
        get_name!(follow item.type_, client, lang.to_string())
      } else {
        item.type_.name.clone()
      });
    }

    // Return types
    result.push(format!(
      "{}:",
      if !fast {
        helpers::get_pokemon_name(&client, &mon_resource, &lang.to_string()).await
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

  #[tokio::test]
  async fn test_types() {
    let client = RustemonClient::default();

    let success: Vec<String> = vec!["Toxel:", "  Electric/Poison"]
      .into_iter()
      .map(|x| x.into())
      .collect();

    for fast in [false, true].into_iter() {
      let pokemon = String::from("toxel");
      let lang = LanguageId::En;
      let recursive = false;

      match print_types(&client, &pokemon, fast, lang, recursive).await {
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
  async fn test_types_recursive() {
    let client = RustemonClient::default();

    let success = vec!["stantler:", "  normal", "wyrdeer:", "  normal/psychic"];
    let pokemon = String::from("stantler");
    let fast = true;
    let lang = LanguageId::En;
    let recursive = true;

    match print_types(&client, &pokemon, fast, lang, recursive).await {
      Ok(s) => assert_eq!(s, success),
      Err(err) => panic!("{}", err.render()),
    }
  }
}
