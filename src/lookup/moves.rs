use crate::get_name;
use crate::utils::args::MoveArgs;
use crate::utils::cli;
use crate::utils::helpers;
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::pokemon;

pub async fn print_moves(
  client: &RustemonClient,
  args: MoveArgs,
) -> Result<Vec<String>, clap::Error> {
  // Create struct to store move
  struct Move {
    name: String,
    level: i64,
  }

  // Create pokemon resource
  let Ok(mon_resource) = pokemon::get_by_name(&args.pokemon.replace(' ', "-"), client).await else {
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

  // Get full learnset
  let mut moves = Vec::new();
  for move_resource in &mon_resource.moves {
    for details in &move_resource.version_group_details {
      if details.move_learn_method.name == "level-up"
        && details.version_group.name == args.vgroup.to_string()
      {
        match args.level {
          Some(x) if details.level_learned_at > x => {},
          _ => {
            moves.push(Move {
              name: if args.fast {
                move_resource.move_.name.clone()
              } else {
                get_name!(follow move_resource.move_, client, args.lang.to_string())
              },
              level: details.level_learned_at,
            });
          },
        }
      }
    }
  }

  // Sort moves by descending level
  moves.sort_by_key(|m| std::cmp::Reverse(m.level));

  // Get current moveset (if requested)
  let mut moves = if args.level.is_some() {
    moves.iter().take(4).collect::<Vec<_>>()
  } else {
    moves.iter().collect::<Vec<_>>()
  };
  moves.reverse();

  // Return moves
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if args.fast {
      mon_resource.name.clone()
    } else {
      helpers::get_pokemon_name(client, &mon_resource, &args.lang.to_string()).await
    }
  ));
  for m in moves {
    result.push(format!(" - {} ({})", m.name, m.level));
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::enums::{LanguageId, VersionGroup};

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
      let args = MoveArgs {
        pokemon: String::from("quaxly"),
        fast: idx == 0,
        lang: LanguageId::En,
        vgroup: VersionGroup::ScarletViolet,
        level: None,
      };

      match print_moves(&client, args).await {
        Ok(res) => assert_eq!(res, vals),
        Err(err) => panic!("{}", err.render()),
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

    let args = MoveArgs {
      pokemon: String::from("quaxly"),
      fast: false,
      lang: LanguageId::En,
      vgroup: VersionGroup::ScarletViolet,
      level: Some(30),
    };

    match print_moves(&client, args).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }
}
