use crate::get_name;
use crate::utils::cli::{self, LanguageId, VersionGroup};
use crate::utils::helpers;
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::*;

pub async fn print_moves(
  client: &RustemonClient,
  pokemon: &str,
  fast: bool,
  lang: LanguageId,
  vgroup: VersionGroup,
  level: Option<i64>,
) -> Result<Vec<String>, clap::Error> {
  // Create pokemon resource
  let mon_resource = match pokemon::get_by_name(&pokemon.replace(" ", "-"), &client).await {
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

  // Create struct to store move
  struct Move {
    name: String,
    level: i64,
  }

  // Get full learnset
  let mut moves = Vec::new();
  for move_resource in mon_resource.moves.iter() {
    for details in move_resource.version_group_details.iter() {
      if details.move_learn_method.name == "level-up"
        && details.version_group.name == vgroup.to_string()
      {
        match level {
          Some(x) if details.level_learned_at > x => {},
          _ => {
            moves.push(Move {
              name: if !fast {
                get_name!(follow move_resource.move_, client, lang.to_string())
              } else {
                move_resource.move_.name.clone()
              },
              level: details.level_learned_at,
            });
          },
        };
      }
    }
  }

  // Sort moves by descending level
  moves.sort_by(|m, n| n.level.cmp(&m.level));

  // Get current moveset (if requested)
  let mut moves = if let Some(_) = level {
    moves.iter().take(4).collect::<Vec<_>>()
  } else {
    moves.iter().collect::<Vec<_>>()
  };
  moves.reverse();

  // Return moves
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast {
      helpers::get_pokemon_name(&client, &mon_resource, &lang.to_string()).await
    } else {
      mon_resource.name.clone()
    }
  ));
  moves
    .iter()
    .for_each(|x| result.push(format!(" - {} ({})", x.name, x.level)));

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_moves() {
    let client = RustemonClient::default();

    let success = vec![
      vec![
        "quaxly:", " - water-gun (1)", " - growl (1)", " - pound (1)", " - work-up (7)",
        " - wing-attack (10)", " - aqua-jet (13)", " - double-hit (17)", " - aqua-cutter (21)",
        " - air-slash (24)", " - focus-energy (28)", " - acrobatics (31)", " - liquidation (35)",
      ],
      vec![
        "Quaxly:", " - Water Gun (1)", " - Growl (1)", " - Pound (1)", " - Work Up (7)",
        " - Wing Attack (10)", " - Aqua Jet (13)", " - Double Hit (17)", " - Aqua Cutter (21)",
        " - Air Slash (24)", " - Focus Energy (28)", " - Acrobatics (31)", " - Liquidation (35)",
      ],
    ];

    for (idx, vals) in success.into_iter().enumerate() {
      let pokemon = String::from("quaxly");
      let fast = idx == 0;
      let lang = LanguageId::En;
      let vgroup = VersionGroup::ScarletViolet;
      let level = None;

      match print_moves(&client, &pokemon, fast, lang, vgroup, level).await {
        Ok(res) => assert_eq!(res, vals),
        Err(err) => err.exit(),
      }
    }
  }

  #[tokio::test]
  async fn test_moves_level() {
    let client = RustemonClient::default();

    let success = vec![
      "Quaxly:", " - Double Hit (17)", " - Aqua Cutter (21)", " - Air Slash (24)",
      " - Focus Energy (28)",
    ];

    let pokemon = String::from("quaxly");
    let level = Some(30);
    let fast = false;
    let vgroup = VersionGroup::ScarletViolet;
    let lang = LanguageId::En;

    match print_moves(&client, &pokemon, fast, lang, vgroup, level).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => err.exit(),
    }
  }
}
