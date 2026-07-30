use crate::get_name;
use crate::utils::args::GenderArgs;
use crate::utils::cli;
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::pokemon_species;

pub async fn print_genders(
  client: &RustemonClient,
  args: GenderArgs,
) -> Result<Vec<String>, clap::Error> {
  // Create pokemon species resource
  let Ok(species) = pokemon_species::get_by_name(&args.pokemon.replace(' ', "-"), client).await
  else {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!("invalid pokemon species: {}", args.pokemon),
    ));
  };

  // Return gender ratio
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if args.fast {
      species.name.clone()
    } else {
      get_name!(species, client, args.lang.to_string())
    }
  ));

  let rate = species.gender_rate as f64 / 8.0 * 100.0;
  if rate < 0.0 {
    result.push(" Genderless".to_string());
  } else {
    result.push(format!(" M: {:>5.1}%", 100.0 - rate));
    result.push(format!(" F: {rate:>5.1}%"));
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::enums::LanguageId;

  #[tokio::test]
  async fn test_genders() {
    let client = RustemonClient::default();

    for fast in [false, true] {
      let args = GenderArgs {
        pokemon: String::from("meowth"),
        fast,
        lang: LanguageId::En,
      };

      match print_genders(&client, args).await {
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
