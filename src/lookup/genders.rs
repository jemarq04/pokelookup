use crate::get_name;
use crate::utils::cli;
use crate::utils::enums::LanguageId;
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::*;

pub async fn print_genders(
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

  // Return gender ratio
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast {
      get_name!(species, client, lang.to_string())
    } else {
      species.name.clone()
    }
  ));
  let rate = species.gender_rate as f64 / 8.0 * 100.0;
  if rate < 0.0 {
    result.push(format!(" Genderless"));
  } else {
    result.push(format!(" M: {:>5.1}%", 100.0 - rate));
    result.push(format!(" F: {:>5.1}%", rate));
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_genders() {
    let client = RustemonClient::default();

    for fast in vec![false, true].into_iter() {
      let pokemon = String::from("meowth");
      let lang = LanguageId::En;

      match print_genders(&client, &pokemon, fast, lang).await {
        Ok(s) => assert_eq!(
          s,
          vec![
            if fast { "meowth:" } else { "Meowth:" },
            " M:  50.0%",
            " F:  50.0%",
          ]
        ),
        Err(err) => panic!("{}", err.render()),
      }
    }
  }
}
