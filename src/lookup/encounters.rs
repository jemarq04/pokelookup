use crate::get_name;
use crate::utils::cli;
use crate::utils::enums::{LanguageId, Version};
use crate::utils::helpers;
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;

pub async fn print_encounters(
  client: &RustemonClient,
  version: Version,
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
    // Get encounter resources
    let encounters = match helpers::follow_encounters(&mon_resource) {
      Ok(x) => x,
      Err(_) => {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!(
            "API error: could not follow url for encounters for {}",
            mon_resource.name
          ),
        ));
      },
    };

    // Get location area names
    let mut encounter_names = Vec::new();
    for enc in encounters.iter() {
      for det in enc.version_details.iter() {
        if det.version.name == version.to_string() {
          encounter_names.push(if !fast {
            get_name!(follow enc.location_area, client, lang.to_string())
          } else {
            enc.location_area.name.clone()
          });
          break;
        }
      }
    }

    // Do not return empty entries
    if encounter_names.len() == 0 {
      continue;
    }

    // Return location areas
    result.push(format!(
      "{}:",
      if !fast {
        helpers::get_pokemon_name(&client, &mon_resource, &lang.to_string()).await
      } else {
        mon_resource.name.clone()
      }
    ));
    encounter_names
      .into_iter()
      .for_each(|name| result.push(format!(" - {name}")));
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_encounters() {
    let client = RustemonClient::default();

    let success = vec![
      vec![
        "machop:",
        " - rock-tunnel-1f",
        " - rock-tunnel-b1f",
        " - kanto-victory-road-2-1f",
        " - kanto-victory-road-2-2f",
        " - kanto-victory-road-2-3f",
        " - mt-ember-area",
        " - mt-ember-cave",
        " - mt-ember-inside",
        " - mt-ember-1f-cave-behind-team-rocket",
      ],
      vec![
        "Machop:",
        " - Rock Tunnel (1F)",
        " - Rock Tunnel (B1F)",
        " - Victory Road 2 (1F)",
        " - Victory Road 2 (2F)",
        " - Victory Road 2 (3F)",
        " - Mount Ember",
        " - Mount Ember (cave)",
        " - Mount Ember (inside)",
        " - Mount Ember (1F, cave behind team rocket)",
      ],
    ];

    for (idx, vals) in success.into_iter().enumerate() {
      let version = Version::Firered;
      let pokemon = String::from("machop");
      let fast = idx == 0;
      let lang = LanguageId::En;
      let recursive = false;

      match print_encounters(&client, version, &pokemon, fast, lang, recursive).await {
        Ok(res) => assert_eq!(res, vals),
        Err(err) => panic!("{}", err.render()),
      }
    }
  }

  #[tokio::test]
  async fn test_encounters_recursive() {
    let client = RustemonClient::default();

    let success = vec![
      "goldeen:",
      " - viridian-city-area",
      " - fuchsia-city-area",
      " - kanto-route-6-area",
      " - kanto-route-22-area",
      " - kanto-route-25-area",
      " - cerulean-cave-1f",
      " - cerulean-cave-b1f",
      " - kanto-route-23-area",
      " - kanto-safari-zone-middle",
      " - kanto-safari-zone-area-1-east",
      " - kanto-safari-zone-area-2-north",
      " - kanto-safari-zone-area-3-west",
      " - berry-forest-area",
      " - icefall-cave-entrance",
      " - cape-brink-area",
      " - ruin-valley-area",
      " - four-island-area",
      "seaking:",
      " - fuchsia-city-area",
      " - kanto-safari-zone-middle",
      " - kanto-safari-zone-area-1-east",
      " - kanto-safari-zone-area-2-north",
      " - kanto-safari-zone-area-3-west",
      " - berry-forest-area",
    ];

    let version = Version::Firered;
    let pokemon = String::from("goldeen");
    let fast = true;
    let lang = LanguageId::En;
    let recursive = true;

    match print_encounters(&client, version, &pokemon, fast, lang, recursive).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }
}
