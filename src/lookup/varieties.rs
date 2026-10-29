use crate::get_name;
use crate::utils::args::ListArgs;
use crate::utils::cli;
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::pokemon_species;

pub async fn print_varieties(
  client: &RustemonClient,
  args: ListArgs,
) -> Result<Vec<String>, clap::Error> {
  // Create pokemon species resource
  let Ok(species) = pokemon_species::get_by_name(&args.pokemon.replace(' ', "-"), client).await
  else {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!("invalid pokemon species: {}", args.pokemon),
    ));
  };

  // Return varieties
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if args.fast {
      species.name.clone()
    } else {
      get_name!(species, client, args.lang.to_string())
    }
  ));
  species
    .varieties
    .iter()
    .for_each(|variety| result.push(format!(" - {}", variety.pokemon.name)));

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::enums::LanguageId;

  #[tokio::test]
  async fn test_varieties() {
    let client = RustemonClient::default();

    for fast in [false, true] {
      let args = ListArgs {
        pokemon: String::from("meowth"),
        fast,
        lang: LanguageId::En,
      };

      match print_varieties(&client, args).await {
        Ok(s) => {
          assert_eq!(
            s,
            vec![
              if fast { "meowth:" } else { "Meowth:" },
              " - meowth",
              " - meowth-alola",
              " - meowth-galar",
              " - meowth-gmax",
            ]
          );
        },
        Err(err) => panic!("{}", err.render()),
      }
    }
  }
}
