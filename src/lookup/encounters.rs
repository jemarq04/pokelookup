use crate::get_name;
use crate::utils::args::EncounterArgs;
use crate::utils::cli;
use crate::utils::enums::Version;
use crate::utils::helpers;
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::model::encounters::EncounterMethod;

pub async fn print_encounters(
  client: &RustemonClient,
  args: EncounterArgs,
) -> Result<Vec<String>, clap::Error> {
  // Determine allowed versions
  let sword_versions = vec![
    Version::Sword,
    Version::TheIsleOfArmorSword,
    Version::TheCrownTundraSword,
  ];
  let shield_versions = vec![
    Version::Shield,
    Version::TheIsleOfArmorShield,
    Version::TheCrownTundraShield,
  ];
  let scarlet_versions = vec![
    Version::Scarlet,
    Version::TheTealMaskScarlet,
    Version::TheIndigoDiskScarlet,
  ];
  let violet_versions = vec![
    Version::Violet,
    Version::TheTealMaskViolet,
    Version::TheIndigoDiskViolet,
  ];

  let mut versions = vec![args.version];
  if args.with_dlc {
    if sword_versions.contains(&args.version) {
      versions = sword_versions;
    } else if shield_versions.contains(&args.version) {
      versions = shield_versions;
    } else if scarlet_versions.contains(&args.version) {
      versions = scarlet_versions;
    } else if violet_versions.contains(&args.version) {
      versions = violet_versions;
    }
  }

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
    // Get encounter resources
    let encounters = match helpers::follow_encounters(mon_resource) {
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
        if versions
          .iter()
          .any(|vers| vers.to_string() == det.version.name)
        {
          let name = if !args.fast {
            get_name!(follow enc.location_area, client, args.lang.to_string())
          } else {
            enc.location_area.name.clone()
          };
          if args.condensed {
            encounter_names.push(name);
          } else {
            let mut temp_details = Vec::new();
            temp_details.push(name);

            let mut encounter_methods: Vec<EncounterMethod> = Vec::new();
            for enc_details in det.encounter_details.iter() {
              if let Ok(encounter_method) = enc_details.method.follow(client).await
                && encounter_methods
                  .iter()
                  .all(|method| method.name != encounter_method.name)
              {
                encounter_methods.push(encounter_method);
              }
            }

            for method in encounter_methods.iter() {
              temp_details.push(format!(
                "   * {}",
                if !args.fast {
                  get_name!(method, client, args.lang.to_string())
                } else {
                  method.name.clone()
                }
              ));
            }
            //temp_details.push(String::from(""));

            encounter_names.push(temp_details.join("\n"));
          }

          break;
        }
      }
    }

    // Do not return empty entries
    if encounter_names.is_empty() {
      continue;
    }

    // Return location areas
    result.push(format!(
      "{}:",
      if !args.fast {
        helpers::get_pokemon_name(client, mon_resource, &args.lang.to_string()).await
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
  use crate::utils::enums::LanguageId;

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
      let args = EncounterArgs {
        version: Version::Firered,
        pokemon: String::from("machop"),
        fast: idx == 0,
        lang: LanguageId::En,
        recursive: false,
        condensed: true,
        with_dlc: false,
      };

      match print_encounters(&client, args).await {
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

    let args = EncounterArgs {
      version: Version::Firered,
      pokemon: String::from("goldeen"),
      fast: true,
      lang: LanguageId::En,
      recursive: true,
      condensed: true,
      with_dlc: false,
    };

    match print_encounters(&client, args).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }

  #[tokio::test]
  async fn test_encounters_full() {
    let client = RustemonClient::default();

    let success = vec![
      "skwovet:",
      " - galar-route-1-area\n   * overworld\n   * walk",
      " - galar-route-2-main\n   * overworld",
      " - galar-route-3-east\n   * berry-trees",
      " - galar-route-4-area\n   * berry-trees",
      " - galar-route-5-area\n   * berry-trees",
      " - dappled-grove-area\n   * berry-trees",
      " - motostoke-riverbank-area\n   * berry-trees",
      " - north-lake-miloch-area\n   * berry-trees",
      " - rolling-fields-main\n   * berry-trees",
      " - slumbering-weald-main\n   * overworld\n   * walk",
      " - watchtower-ruins-area\n   * berry-trees",
      " - motostoke-pokemon-center\n   * npc-trade",
      " - rolling-fields-max-den-a\n   * max-raid",
      " - east-lake-axewell-max-den-c\n   * max-raid",
      " - motostoke-riverbank-max-den-a\n   * max-raid",
      " - bridge-field-max-den-e\n   * max-raid",
      " - stony-wilderness-max-den-a\n   * max-raid",
    ];

    let args = EncounterArgs {
      version: Version::Sword,
      pokemon: String::from("skwovet"),
      fast: true,
      lang: LanguageId::En,
      recursive: false,
      condensed: false,
      with_dlc: false,
    };

    match print_encounters(&client, args).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }
}
