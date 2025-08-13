use crate::get_name;
use crate::utils::cli::{self, LanguageId};
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::*;

pub async fn print_varieties(
  client: &RustemonClient,
  pokemon: &str,
  fast: bool,
  lang: LanguageId,
) -> Result<Vec<String>, clap::Error> {
  // Create pokemon species resource
  let species = match pokemon_species::get_by_name(&pokemon.replace(' ', "-"), &client).await {
    Ok(x) => x,
    Err(_) => {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        format!("invalid pokemon species: {pokemon}"),
      ));
    },
  };

  // Return varieties
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast {
      get_name!(species, client, lang.to_string())
    } else {
      species.name.clone()
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

  #[tokio::test]
  async fn test_varieties() {
    let client = RustemonClient::default();

    for fast in vec![false, true].into_iter() {
      let pokemon = String::from("meowth");
      let lang = LanguageId::En;

      match print_varieties(&client, &pokemon, fast, lang).await {
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
        Err(err) => err.exit(),
      }
    }
  }
}
