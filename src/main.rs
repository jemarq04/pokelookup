mod utils;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use futures::future;
use itertools::izip;
use rustemon::Follow;
use rustemon::client::{CACacheManager, RustemonClient, RustemonClientBuilder};
use rustemon::pokemon::*;
use utils::*;

#[tokio::main]
async fn main() {
  let mut args = Args::parse();

  // Create cache directory for API calls
  if let None = args.cache_dir {
    args.cache_dir = match std::env::home_dir() {
      Some(path) => Some(format!("{}/.cache/pokelookup", path.display()).into()),
      None => None,
    }
  }
  let client = match args.cache_dir {
    Some(path) => {
      match RustemonClientBuilder::default()
        .with_manager(CACacheManager { path })
        .try_build()
      {
        Ok(cl) => cl,
        Err(_) => {
          eprintln!("warning: cache directory set to cache manager default");
          RustemonClient::default()
        },
      }
    },
    None => {
      eprintln!("warning: cache directory set to cache manager default");
      RustemonClient::default()
    },
  };

  // Call the appropriate subcommand for results
  let result = match args.command {
    SubArgs::ListCmd { .. } => print_varieties(&args.command, &client).await,
    SubArgs::TypeCmd { .. } => print_types(&args.command, &client).await,
    SubArgs::AbilityCmd { .. } => print_abilities(&args.command, &client).await,
    SubArgs::MoveCmd { .. } => print_moves(&args.command, &client).await,
    SubArgs::EggCmd { .. } => print_eggs(&args.command, &client).await,
    SubArgs::GenderCmd { .. } => print_genders(&args.command, &client).await,
    SubArgs::EncounterCmd { .. } => print_encounters(&args.command, &client).await,
    SubArgs::EvolutionCmd { .. } => print_evolutions(&args.command, &client).await,
    SubArgs::MatchupCmd { .. } => print_matchups(&args.command, &client).await,
  };

  // Handle output
  match result {
    Ok(s) if s.len() == 0 => println!("No results found."),
    Ok(s) => s.iter().for_each(|x| println!("{}", x)),
    Err(err) => err.exit(),
  };
}

async fn print_varieties(
  args: &SubArgs,
  client: &RustemonClient,
) -> Result<Vec<String>, clap::Error> {
  let SubArgs::ListCmd { pokemon, fast, .. } = args else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create pokemon species resource
  let species = match pokemon_species::get_by_name(&pokemon, &client).await {
    Ok(x) => x,
    Err(_) => {
      return Err(Args::command().error(
        ErrorKind::InvalidValue,
        format!("invalid pokemon species: {pokemon}"),
      ));
    },
  };

  // Return varieties
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast && let Ok(name) = get_name(&client, &species.names, "en").await {
      name
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

async fn print_types(args: &SubArgs, client: &RustemonClient) -> Result<Vec<String>, clap::Error> {
  let SubArgs::TypeCmd {
    pokemon,
    fast,
    recursive,
    ..
  } = args
  else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create pokemon resources
  let resources = match get_pokemon_from_chain(&client, &pokemon, *recursive).await {
    Ok(x) => x,
    Err(_) => {
      let valid = Args::command().get_styles().get_valid().clone();
      let err = Args::command().error(
        ErrorKind::InvalidValue,
        format!(
          "invalid pokemon: {pokemon}\n\n{valid}tip:{valid:#} try running '{} list {pokemon}'",
          Args::command().get_name()
        ),
      );
      return Err(err);
    },
  };

  // Iterate over all requested pokemon
  let mut result = Vec::new();
  for mon_resource in resources.iter() {
    // Get type resources
    let types = match future::try_join_all(
      mon_resource
        .types
        .iter()
        .map(async |t| t.type_.follow(&client).await),
    )
    .await
    {
      Ok(x) => x,
      Err(_) => {
        return Err(Args::command().error(
          ErrorKind::InvalidValue,
          format!(
            "API error: could not retrieve types for {}",
            mon_resource.name,
          ),
        ));
      },
    };

    // Get type names
    let mut type_names = Vec::new();
    for type_ in types.iter() {
      if !fast && let Ok(name) = get_name(&client, &type_.names, "en").await {
        type_names.push(name);
      } else {
        type_names.push(type_.name.clone());
      }
    }

    // Return types
    result.push(format!(
      "{}:",
      if !fast && let Ok(name) = get_pokemon_name(&client, &mon_resource, "en").await {
        name
      } else {
        mon_resource.name.clone()
      }
    ));
    result.push(format!("  {}", type_names.join("/")));
  }

  Ok(result)
}

async fn print_abilities(
  args: &SubArgs,
  client: &RustemonClient,
) -> Result<Vec<String>, clap::Error> {
  let SubArgs::AbilityCmd {
    pokemon,
    fast,
    recursive,
    ..
  } = args
  else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create pokemon resources
  let resources = match get_pokemon_from_chain(&client, &pokemon, *recursive).await {
    Ok(x) => x,
    Err(_) => {
      let valid = Args::command().get_styles().get_valid().clone();
      let err = Args::command().error(
        ErrorKind::InvalidValue,
        format!(
          "invalid pokemon: {pokemon}\n\n{valid}tip:{valid:#} try running '{} list {pokemon}'",
          Args::command().get_name()
        ),
      );
      return Err(err);
    },
  };

  // Create struct to store ability
  struct Ability {
    hidden: bool,
    ability: rustemon::model::pokemon::Ability,
  }

  // Iterate over all requested pokemon
  let mut result = Vec::new();
  for mon_resource in resources.iter() {
    // Get ability resources
    let abilities = match future::try_join_all(mon_resource.abilities.iter().map(async |a| {
      match a.ability.follow(&client).await {
        Ok(x) => Ok(Ability {
          hidden: a.is_hidden,
          ability: x,
        }),
        Err(_) => Err(()),
      }
    }))
    .await
    {
      Ok(x) => x,
      Err(_) => {
        return Err(Args::command().error(
          ErrorKind::InvalidValue,
          format!(
            "API error: could not retrieve abilities for {}",
            mon_resource.name,
          ),
        ));
      },
    };

    // Get ability names
    let mut names = Vec::new();
    for ab in abilities.into_iter() {
      if !fast && let Ok(name) = get_name(&client, &ab.ability.names, "en").await {
        names.push(name + if ab.hidden { " (Hidden)" } else { "" });
      } else {
        names.push(ab.ability.name.clone() + if ab.hidden { " (hidden)" } else { "" });
      }
    }

    // Return abilities
    result.push(format!(
      "{}:",
      if !fast && let Ok(name) = get_pokemon_name(&client, &mon_resource, "en").await {
        name
      } else {
        mon_resource.name.clone()
      }
    ));
    names
      .iter()
      .enumerate()
      .for_each(|x| result.push(format!(" {}. {}", x.0 + 1, x.1)));
  }

  Ok(result)
}

async fn print_moves(args: &SubArgs, client: &RustemonClient) -> Result<Vec<String>, clap::Error> {
  let SubArgs::MoveCmd {
    pokemon,
    fast,
    vgroup,
    level,
    ..
  } = args
  else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create pokemon resource
  let mon_resource = match pokemon::get_by_name(&pokemon, &client).await {
    Ok(x) => x,
    Err(_) => {
      let valid = Args::command().get_styles().get_valid().clone();
      let err = Args::command().error(
        ErrorKind::InvalidValue,
        format!(
          "invalid pokemon: {pokemon}\n\n{valid}tip:{valid:#} try running '{} list {pokemon}'",
          Args::command().get_name()
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
        match *level {
          Some(x) if details.level_learned_at > x => {},
          _ => {
            if !fast
              && let Ok(x) = move_resource.move_.follow(&client).await
              && let Ok(name) = get_name(&client, &x.names, "en").await
            {
              moves.push(Move {
                name,
                level: details.level_learned_at,
              });
            } else {
              moves.push(Move {
                name: move_resource.move_.name.clone(),
                level: details.level_learned_at,
              });
            }
          },
        };
      }
    }
  }

  // Sort moves by descending level
  moves.sort_by(|m, n| n.level.cmp(&m.level));

  // Get current moveset (if requested)
  let mut moves = if let Some(_) = *level {
    moves.iter().take(4).collect::<Vec<_>>()
  } else {
    moves.iter().collect::<Vec<_>>()
  };
  moves.reverse();

  // Return moves
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast && let Ok(name) = get_pokemon_name(&client, &mon_resource, "en").await {
      name
    } else {
      mon_resource.name.clone()
    }
  ));
  moves
    .iter()
    .for_each(|x| result.push(format!(" - {} ({})", x.name, x.level)));

  Ok(result)
}

async fn print_eggs(args: &SubArgs, client: &RustemonClient) -> Result<Vec<String>, clap::Error> {
  let SubArgs::EggCmd { pokemon, fast, .. } = args else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create pokemon species resource
  let species = match pokemon_species::get_by_name(&pokemon, &client).await {
    Ok(x) => x,
    Err(_) => {
      return Err(Args::command().error(
        ErrorKind::InvalidValue,
        format!("invalid pokemon species: {pokemon}"),
      ));
    },
  };

  // Get egg group resources
  let eggs = match future::try_join_all(
    species
      .egg_groups
      .iter()
      .map(async |g| g.follow(&client).await),
  )
  .await
  {
    Ok(x) => x,
    Err(_) => {
      return Err(Args::command().error(
        ErrorKind::InvalidValue,
        format!(
          "API error: could not retrieve egg groups for {}",
          species.name,
        ),
      ));
    },
  };

  // Get egg group names
  let mut egg_names = Vec::new();
  for egg in eggs.iter() {
    if !fast && let Ok(name) = get_name(&client, &egg.names, "en").await {
      egg_names.push(name);
    } else {
      egg_names.push(egg.name.clone());
    }
  }

  // Return egg groups
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast && let Ok(name) = get_name(&client, &species.names, "en").await {
      name
    } else {
      species.name.clone()
    }
  ));
  egg_names
    .iter()
    .for_each(|name| result.push(format!(" - {name}")));

  Ok(result)
}

async fn print_genders(
  args: &SubArgs,
  client: &RustemonClient,
) -> Result<Vec<String>, clap::Error> {
  let SubArgs::GenderCmd { pokemon, fast, .. } = args else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create pokemon species resource
  let species = match pokemon_species::get_by_name(&pokemon, &client).await {
    Ok(x) => x,
    Err(_) => {
      return Err(Args::command().error(
        ErrorKind::InvalidValue,
        format!("invalid pokemon species: {pokemon}"),
      ));
    },
  };

  // Return gender ratio
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast && let Ok(name) = get_name(&client, &species.names, "en").await {
      name
    } else {
      species.name.clone()
    }
  ));
  let rate = species.gender_rate as f64 / 8.0 * 100.0;
  if rate < 0.0 {
    result.push(format!(" Genderless"));
  } else {
    result.push(format!(" M: {:>5.1}", 100.0 - rate));
    result.push(format!(" F: {:>5.1}", rate));
  }

  Ok(result)
}

async fn print_encounters(
  args: &SubArgs,
  client: &RustemonClient,
) -> Result<Vec<String>, clap::Error> {
  let SubArgs::EncounterCmd {
    version,
    pokemon,
    fast,
    recursive,
    ..
  } = args
  else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create pokemon resources
  let resources = match get_pokemon_from_chain(&client, &pokemon, *recursive).await {
    Ok(x) => x,
    Err(_) => {
      let valid = Args::command().get_styles().get_valid().clone();
      let err = Args::command().error(
        ErrorKind::InvalidValue,
        format!(
          "invalid pokemon: {pokemon}\n\n{valid}tip:{valid:#} try running '{} list {pokemon}'",
          Args::command().get_name()
        ),
      );
      return Err(err);
    },
  };

  // Iterate over all requested pokemon
  let mut result = Vec::new();
  for mon_resource in resources.iter() {
    // Get encounter resources
    let encounters = match follow_encounters(&mon_resource) {
      Ok(x) => x,
      Err(_) => {
        return Err(Args::command().error(
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
          encounter_names.push(
            if !fast
              && let Ok(x) = enc.location_area.follow(&client).await
              && let Ok(name) = get_name(&client, &x.names, "en").await
            {
              name
            } else {
              enc.location_area.name.clone()
            },
          );
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
      if !fast && let Ok(name) = get_pokemon_name(&client, &mon_resource, "en").await {
        name
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

async fn print_evolutions(
  args: &SubArgs,
  client: &RustemonClient,
) -> Result<Vec<String>, clap::Error> {
  let SubArgs::EvolutionCmd {
    pokemon,
    fast,
    secret,
    ..
  } = args
  else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create pokemon species resource
  let species = match pokemon_species::get_by_name(&pokemon, &client).await {
    Ok(x) => x,
    Err(_) => {
      return Err(Args::command().error(
        ErrorKind::InvalidValue,
        format!("invalid pokemon species: {pokemon}"),
      ));
    },
  };

  // Iterate over evolution chain, if present
  let mut result: Vec<String> = Vec::new();
  if let Some(chain_resource) = species.evolution_chain {
    // Get evolution chain resource
    let chain = match chain_resource.follow(&client).await {
      Ok(x) => x,
      Err(_) => {
        return Err(Args::command().error(
          ErrorKind::InvalidValue,
          format!(
            "API error: could not retrieve evolution chain for {}",
            species.name,
          ),
        ));
      },
    };

    if chain.chain.evolves_to.len() == 0 {
      // Record first species name
      result.push(get_evolution_name(&client, &chain.chain.species, "en", *fast, *secret).await);
    }

    for evo1 in chain.chain.evolves_to.iter() {
      if evo1.evolution_details.len() == 0 {
        result.push(format!(
          "{} -> ??? -> {}",
          get_evolution_name(&client, &chain.chain.species, "en", *fast, *secret).await,
          get_evolution_name(&client, &evo1.species, "en", *fast, *secret).await,
        ));
      } else {
        for method1 in evo1.evolution_details.iter() {
          result.push(format!(
            "{} -> {}",
            get_evolution_name(&client, &chain.chain.species, "en", *fast, *secret).await,
            if !fast
              && let Ok(trigger) = method1.trigger.follow(&client).await
              && let Ok(name) = get_name(&client, &trigger.names, "en").await
            {
              name
            } else {
              method1.trigger.name.clone()
            },
          ));

          if let Some(details) = get_evolution_details(&client, &method1, "en", *fast).await {
            result
              .last_mut()
              .unwrap()
              .push_str(&format!(" ({details})"));
          }

          result.last_mut().unwrap().push_str(&format!(
            " -> {}",
            get_evolution_name(&client, &evo1.species, "en", *fast, *secret).await,
          ));

          let mut first_evo2 = true;
          let curr_steps = result.last().unwrap().clone();
          for evo2 in evo1.evolves_to.iter() {
            for method2 in evo2.evolution_details.iter() {
              let mut temp_steps: String = format!(
                " -> {}",
                if !fast
                  && let Ok(trigger) = method2.trigger.follow(&client).await
                  && let Ok(name) = get_name(&client, &trigger.names, "en").await
                {
                  name
                } else {
                  method2.trigger.name.clone()
                },
              );

              if let Some(details) = get_evolution_details(&client, &method2, "en", *fast).await {
                result
                  .last_mut()
                  .unwrap()
                  .push_str(&format!(" ({details})"));
              }

              temp_steps.push_str(&format!(
                " -> {}",
                get_evolution_name(&client, &evo2.species, "en", *fast, *secret).await,
              ));

              if first_evo2 {
                result.last_mut().unwrap().push_str(&temp_steps);
                first_evo2 = false;
              } else {
                result.push(format!("{}{}", curr_steps, temp_steps));
              }
            }
          }
        }
      }
    }
  } else {
    // No chain found => record species name to final result
    result.push(if !secret {
      if !fast && let Ok(name) = get_name(&client, &species.names, "en").await {
        name
      } else {
        species.name.clone()
      }
    } else {
      String::from("MON")
    });
  }

  Ok(result)
}

async fn print_matchups(
  args: &SubArgs,
  client: &RustemonClient,
) -> Result<Vec<String>, clap::Error> {
  let SubArgs::MatchupCmd {
    primary, secondary, ..
  } = args
  else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Get type resources
  let primary = match type_::get_by_name(&primary.to_string(), &client).await {
    Ok(x) => x,
    Err(_) => {
      return Err(Args::command().error(
        ErrorKind::InvalidValue,
        format!("API error: could not retrieve type {primary}"),
      ));
    },
  };
  let secondary = match secondary {
    Some(t) => match type_::get_by_name(&t.to_string(), &client).await {
      Ok(x) => Some(x),
      Err(_) => {
        return Err(Args::command().error(
          ErrorKind::InvalidValue,
          format!("API error: could not retrieve type {t}"),
        ));
      },
    },
    None => None,
  };

  // Get matchups from other types
  let mut no_damage_from = Vec::new();
  let mut half_damage_from = Vec::new();
  let mut double_damage_from = Vec::new();
  let mut quarter_damage_from = Vec::new();
  let mut quad_damage_from = Vec::new();

  for other_type in primary.damage_relations.no_damage_from.iter() {
    no_damage_from.push(Type::to_title(&other_type.name).unwrap());
  }
  for other_type in primary.damage_relations.half_damage_from.iter() {
    half_damage_from.push(Type::to_title(&other_type.name).unwrap());
  }
  for other_type in primary.damage_relations.double_damage_from.iter() {
    double_damage_from.push(Type::to_title(&other_type.name).unwrap());
  }
  if let Some(ref second) = secondary {
    for other_type in second.damage_relations.no_damage_from.iter() {
      let name = Type::to_title(&other_type.name).unwrap();
      if let Some(idx) = half_damage_from.iter().position(|x| *x == name) {
        half_damage_from.remove(idx);
        no_damage_from.push(name.clone());
      } else if let Some(idx) = double_damage_from.iter().position(|x| *x == name) {
        double_damage_from.remove(idx);
        no_damage_from.push(name.clone());
      } else if let None = no_damage_from.iter().position(|x| *x == name) {
        no_damage_from.push(name.clone());
      }
    }
    for other_type in second.damage_relations.half_damage_from.iter() {
      let name = Type::to_title(&other_type.name).unwrap();
      if let Some(idx) = half_damage_from.iter().position(|x| *x == name) {
        quarter_damage_from.push(name.clone());
        half_damage_from.remove(idx);
      } else if let Some(idx) = double_damage_from.iter().position(|x| *x == name) {
        double_damage_from.remove(idx);
      } else if let None = no_damage_from.iter().position(|x| *x == name) {
        half_damage_from.push(name.clone());
      }
    }
    for other_type in second.damage_relations.double_damage_from.iter() {
      let name = Type::to_title(&other_type.name).unwrap();
      if let Some(idx) = half_damage_from.iter().position(|x| *x == name) {
        half_damage_from.remove(idx);
      } else if let Some(idx) = double_damage_from.iter().position(|x| *x == name) {
        quad_damage_from.push(name.clone());
        double_damage_from.remove(idx);
      } else if let None = no_damage_from.iter().position(|x| *x == name) {
        double_damage_from.push(name.clone());
      }
    }
  }

  // Bring all vectors to the same size
  let maxlen = itertools::max(vec![
    no_damage_from.len(),
    half_damage_from.len(),
    double_damage_from.len(),
  ])
  .unwrap();
  while no_damage_from.len() < maxlen {
    no_damage_from.push(String::new());
  }
  while half_damage_from.len() < maxlen {
    half_damage_from.push(String::new());
  }
  while double_damage_from.len() < maxlen {
    double_damage_from.push(String::new());
  }
  while quarter_damage_from.len() < maxlen {
    quarter_damage_from.push(String::new());
  }
  while quad_damage_from.len() < maxlen {
    quad_damage_from.push(String::new());
  }

  // Return type matchups
  let mut result = Vec::new();
  match secondary {
    None => {
      result.push(format!("{:^8} {:^8} {:^8}", "*0", "*0.5", "*2"));
      result.push(format!("{:-<8} {:-<8} {:-<8}", "", "", ""));
      for (no_dmg, half_dmg, double_dmg) in
        izip!(&no_damage_from, &half_damage_from, &double_damage_from)
      {
        result.push(format!("{:<8} {:<8} {:<8}", no_dmg, half_dmg, double_dmg));
      }
    },
    Some(_) => {
      result.push(format!(
        "{:^8} {:^8} {:^8} {:^8} {:^8}",
        "*0", "*0.25", "*0.5", "*2", "*4"
      ));
      result.push(format!(
        "{:-<8} {:-<8} {:-<8} {:-<8} {:-<8}",
        "", "", "", "", ""
      ));
      for (no_dmg, quarter_dmg, half_dmg, double_dmg, quad_dmg) in izip!(
        &no_damage_from, &quarter_damage_from, &half_damage_from, &double_damage_from,
        &quad_damage_from
      ) {
        result.push(format!(
          "{:<8} {:<8} {:<8} {:<8} {:<8}",
          no_dmg, quarter_dmg, half_dmg, double_dmg, quad_dmg
        ));
      }
    },
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_varieties() {
    let client = RustemonClient::default();

    for fast in vec![false, true].into_iter() {
      let args = SubArgs::ListCmd {
        pokemon: String::from("meowth"),
        fast,
      };

      match print_varieties(&args, &client).await {
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

  #[tokio::test]
  async fn test_types() {
    let client = RustemonClient::default();

    let success: Vec<String> = vec!["Toxel:", "  Electric/Poison"]
      .into_iter()
      .map(|x| x.into())
      .collect();

    for fast in [false, true].into_iter() {
      let args = SubArgs::TypeCmd {
        pokemon: String::from("toxel"),
        fast,
        recursive: false,
      };

      match print_types(&args, &client).await {
        Ok(s) => assert_eq!(
          s,
          if fast {
            success.iter().map(|x| x.to_lowercase()).collect()
          } else {
            success.clone()
          }
        ),
        Err(err) => err.exit(),
      }
    }
  }

  #[tokio::test]
  async fn test_types_recursive() {
    let client = RustemonClient::default();

    let success = vec!["stantler:", "  normal", "wyrdeer:", "  normal/psychic"];
    let args = SubArgs::TypeCmd {
      pokemon: String::from("stantler"),
      fast: true,
      recursive: true,
    };
    match print_types(&args, &client).await {
      Ok(s) => assert_eq!(s, success),
      Err(err) => err.exit(),
    }
  }

  #[tokio::test]
  async fn test_abilities() {
    let client = RustemonClient::default();

    let success: Vec<String> = vec!["Toxel:", " 1. Rattled", " 2. Static", " 3. Klutz (Hidden)"]
      .into_iter()
      .map(|x| x.into())
      .collect();

    for fast in [false, true].into_iter() {
      let args = SubArgs::AbilityCmd {
        pokemon: String::from("toxel"),
        fast,
        recursive: false,
      };

      match print_abilities(&args, &client).await {
        Ok(s) => assert_eq!(
          s,
          if fast {
            success.iter().map(|x| x.to_lowercase()).collect()
          } else {
            success.clone()
          }
        ),
        Err(err) => err.exit(),
      }
    }
  }

  #[tokio::test]
  async fn test_abilities_recursive() {
    let client = RustemonClient::default();

    let success = vec![
      "Stantler:", " 1. Intimidate", " 2. Frisk", " 3. Sap Sipper (Hidden)", "Wyrdeer:",
      " 1. Intimidate", " 2. Frisk", " 3. Sap Sipper (Hidden)",
    ];

    let args = SubArgs::AbilityCmd {
      pokemon: String::from("stantler"),
      fast: false,
      recursive: true,
    };

    match print_abilities(&args, &client).await {
      Ok(s) => assert_eq!(s, success),
      Err(err) => err.exit(),
    }
  }

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
      let args = SubArgs::MoveCmd {
        pokemon: String::from("quaxly"),
        vgroup: VersionGroup::ScarletViolet,
        level: None,
        fast: idx == 0,
      };

      match print_moves(&args, &client).await {
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

    let args = SubArgs::MoveCmd {
      pokemon: String::from("quaxly"),
      vgroup: VersionGroup::ScarletViolet,
      level: Some(30),
      fast: false,
    };

    match print_moves(&args, &client).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => err.exit(),
    }
  }

  #[tokio::test]
  async fn test_eggs() {
    let client = RustemonClient::default();

    let success = vec![
      vec!["stantler:", " - ground"],
      vec!["Stantler:", " - Field"],
    ];

    for (idx, vals) in success.into_iter().enumerate() {
      let args = SubArgs::EggCmd {
        pokemon: String::from("stantler"),
        fast: idx == 0,
      };

      match print_eggs(&args, &client).await {
        Ok(res) => assert_eq!(res, vals),
        Err(err) => err.exit(),
      }
    }
  }
  #[tokio::test]
  async fn test_genders() {
    let client = RustemonClient::default();

    for fast in vec![false, true].into_iter() {
      let args = SubArgs::GenderCmd {
        pokemon: String::from("meowth"),
        fast,
      };

      match print_genders(&args, &client).await {
        Ok(s) => assert_eq!(
          s,
          vec![
            if fast { "meowth:" } else { "Meowth:" },
            " M:  50.0",
            " F:  50.0",
          ]
        ),
        Err(err) => err.exit(),
      }
    }
  }

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
      let args = SubArgs::EncounterCmd {
        version: Version::Firered,
        pokemon: String::from("machop"),
        recursive: false,
        fast: idx == 0,
      };

      match print_encounters(&args, &client).await {
        Ok(res) => assert_eq!(res, vals),
        Err(err) => err.exit(),
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

    let args = SubArgs::EncounterCmd {
      version: Version::Firered,
      pokemon: String::from("goldeen"),
      fast: true,
      recursive: true,
    };

    match print_encounters(&args, &client).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => err.exit(),
    }
  }

  #[tokio::test]
  async fn test_matchups() {
    let client = RustemonClient::default();

    let success = vec![
      "   *0      *0.5      *2   ", "-------- -------- --------", "Dragon   Fighting Poison  ",
      "         Bug      Steel   ", "         Dark             ",
    ];

    let args = SubArgs::MatchupCmd {
      primary: Type::Fairy,
      secondary: None,
    };

    match print_matchups(&args, &client).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => err.exit(),
    }
  }

  #[tokio::test]
  async fn test_matchups_dual() {
    let client = RustemonClient::default();

    let success = vec![
      "   *0     *0.25     *0.5      *2       *4   ",
      "-------- -------- -------- -------- --------",
      "Electric          Flying   Ground           ",
      "                  Steel    Water            ",
      "                  Poison   Grass            ",
      "                  Rock     Ice              ",
    ];

    let args = SubArgs::MatchupCmd {
      primary: Type::Electric,
      secondary: Some(Type::Ground),
    };

    match print_matchups(&args, &client).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => err.exit(),
    }
  }
}
