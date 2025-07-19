use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use futures::future;
use rustemon::Follow;
use rustemon::pokemon::*;

/// Look up pokemon details using PokeAPI using the 'rustemon' wrapper. Note that sometimes pokemon need to be listed
/// with their forms if the form is distinct enough (e.g. pumkaboo-small or toxtricity-amped). These varieties can be
/// listed using the 'list' subcommand.
#[derive(Parser, Debug)]
#[command(long_about)]
struct Args {
  #[arg(long, hide = true)]
  test: bool,

  #[command(subcommand)]
  command: SubArgs,
}

#[derive(Subcommand, Debug)]
enum SubArgs {
  /// Look up the varieties of a given pokemon.
  #[command(
    name = "list",
    about = "look up the varieties of a pokemon",
    long_about
  )]
  ListCmd {
    #[arg(help = "name of pokemon species")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,
  },

  /// Look up the type(s) of a given pokemon.
  #[command(name = "types", about = "look up the types of a pokemon", long_about)]
  TypeCmd {
    #[arg(help = "name of pokemon")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,

    #[arg(short, help = "recursively check evolution chain")]
    recursive: bool,
  },

  /// Look up the abilities of a given pokemon. If the ability is a hidden ability, it will be
  /// marked accordingly.
  #[command(
    name = "abilities",
    about = "look up the abilities of a pokemon",
    long_about
  )]
  AbilityCmd {
    #[arg(help = "name of pokemon")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,

    #[arg(short, help = "recursively check evolution chain")]
    recursive: bool,
  },

  /// Look up the level-up moveset of a given pokemon. If a level is provided
  /// then the four moves at or below the given level are provided. By default, this will
  /// retrieve the moveset from the Scarlet/Violet version group.
  #[command(
    name = "moves",
    about = "look up the level-up movesets of a pokemon",
    long_about
  )]
  MoveCmd {
    #[arg(help = "name of pokemon")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,

    #[arg(value_enum, short, long, default_value_t=VersionGroup::ScarletViolet,
            hide_possible_values=true, help="version group name")]
    vgroup: VersionGroup,

    #[arg(short, long, help = "request default moveset at given level")]
    level: Option<i64>,
  },

  /// Look up the egg groups of a given pokemon species.
  #[command(
    name = "eggs",
    about = "look up the egg groups of a pokemon",
    long_about
  )]
  EggCmd {
    #[arg(help = "name of pokemon species")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,
  },

  /// Look up the gender ratio of a given pokemon species.
  #[command(
    name = "genders",
    about = "look up the gender ratio of a pokemon",
    long_about
  )]
  GenderCmd {
    #[arg(help = "name of pokemon species")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,
  },

  /// Look up the encounters for a given pokemon and version.
  #[command(
    name = "encounters",
    about = "look up encounters for a pokemon",
    long_about
  )]
  EncounterCmd {
    #[arg(help = "name of version")]
    version: String,

    #[arg(help = "name of pokemon")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,

    #[arg(short, help = "recursively check evolution chain")]
    recursive: bool,
  },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
enum VersionGroup {
  RedBlue,
  Yellow,
  GoldSilver,
  Crystal,
  RubySapphire,
  Emerald,
  FireredLeafgreen,
  Colusseum,
  #[value(alias = "xd")]
  XD,
  DiamondPearl,
  Platinum,
  HeartgoldSoulsilver,
  BlackWhite,
  Black2White2,
  XY,
  OmegaRubyAlphaSapphire,
  SunMoon,
  UltraSunUltraMoon,
  LetsGoPikachuLetsGoEevee,
  SwordShield,
  TheIsleOfArmor,
  TheCrownTundra,
  BrilliantDiamondShiningPearl,
  LegendsArceus,
  ScarletViolet,
  TheTealMask,
  TheIndigoDisk,
}
impl std::fmt::Display for VersionGroup {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self
      .to_possible_value()
      .expect("no values are skipped")
      .get_name()
      .fmt(f)
  }
}

#[tokio::main]
async fn main() {
  let args = Args::parse();
  if args.test {
    println!("{:?}", args);
  }

  let result = match args.command {
    SubArgs::ListCmd { .. } => print_varieties(&args.command).await,
    SubArgs::TypeCmd { .. } => print_types(&args.command).await,
    SubArgs::AbilityCmd { .. } => print_abilities(&args.command).await,
    SubArgs::MoveCmd { .. } => print_moves(&args.command).await,
    SubArgs::EggCmd { .. } => print_eggs(&args.command).await,
    SubArgs::GenderCmd { .. } => print_genders(&args.command).await,
    SubArgs::EncounterCmd { .. } => print_encounters(&args.command).await,
  };

  match result {
    Ok(s) => s.iter().for_each(|x| println!("{}", x)),
    Err(err) => err.exit(),
  };
}

async fn get_name(
  client: &rustemon::client::RustemonClient,
  names: &Vec<rustemon::model::resource::Name>,
  lang: &str,
) -> Result<String, ()> {
  for n in names.iter() {
    if let Ok(x) = n.language.follow(&client).await
      && x.name == lang
    {
      return Ok(n.name.clone());
    }
  }
  Err(())
}

async fn get_pokemon_from_chain(
  client: &rustemon::client::RustemonClient,
  pokemon: &str,
  recursive: bool,
) -> Result<Vec<rustemon::model::pokemon::Pokemon>, ()> {
  let mut result = Vec::new();
  let pokemon = match pokemon::get_by_name(pokemon, &client).await {
    Ok(x) => x,
    Err(_) => return Err(()),
  };

  if recursive {
    let species = match pokemon.species.follow(&client).await {
      Ok(x) => x,
      Err(_) => return Err(()),
    };
    if let Some(chain) = species.evolution_chain {
      let chain = match chain.follow(&client).await {
        Ok(x) => x.chain,
        Err(_) => return Err(()),
      };
      if let Ok(x) = pokemon::get_by_name(&chain.species.name, &client).await {
        result.push(x);
      }
      for evo1 in chain.evolves_to.iter() {
        if let Ok(x) = pokemon::get_by_name(&evo1.species.name, &client).await {
          result.push(x);
        }
        for evo2 in evo1.evolves_to.iter() {
          if let Ok(x) = pokemon::get_by_name(&evo2.species.name, &client).await {
            result.push(x);
          }
        }
      }
    }
  } else {
    result.push(pokemon);
  }

  Ok(result)
}

async fn print_varieties(args: &SubArgs) -> Result<Vec<String>, clap::error::Error> {
  let SubArgs::ListCmd { pokemon, fast, .. } = args else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create client
  let client = rustemon::client::RustemonClient::default();

  // Create pokemon resources
  let species_resource = match pokemon_species::get_by_name(&pokemon, &client).await {
    Ok(x) => x,
    Err(_) => panic!("error: could not find pokemon species {}", pokemon),
  };

  // Print varieties
  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast && let Ok(name) = get_name(&client, &species_resource.names, "en").await {
      name
    } else {
      species_resource.name.clone()
    }
  ));
  species_resource
    .varieties
    .iter()
    .for_each(|x| result.push(format!(" - {}", x.pokemon.name)));

  Ok(result)
}

async fn print_types(args: &SubArgs) -> Result<Vec<String>, clap::error::Error> {
  let SubArgs::TypeCmd {
    pokemon,
    fast,
    recursive,
    ..
  } = args
  else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create client
  let client = rustemon::client::RustemonClient::default();

  // Create pokemon resources
  let resources = match get_pokemon_from_chain(&client, &pokemon, *recursive).await {
    Ok(x) => x,
    Err(_) => panic!("error: could not find pokemon {}", pokemon),
  };

  let mut result = Vec::new();
  for mon_resource in resources.iter() {
    // Get types
    let types = match future::try_join_all(
      mon_resource
        .types
        .iter()
        .map(async |t| t.type_.follow(&client).await),
    )
    .await
    {
      Ok(x) => x,
      Err(_) => panic!("error: could not retrive types for pokemon {}", pokemon),
    };

    // Print English names
    let types = if *fast {
      types.into_iter().map(|t| t.name).collect()
    } else {
      match future::try_join_all(
        types
          .into_iter()
          .map(|t| t.names)
          .map(async |names| get_name(&client, &names, "en").await),
      )
      .await
      {
        Ok(x) => x,
        Err(_) => panic!("error: could not find English names for types"),
      }
    };

    result.push(format!(
      "{}:",
      if !fast
        && let Ok(species) = mon_resource.species.follow(&client).await
        && let Ok(name) = get_name(&client, &species.names, "en").await
      {
        name
      } else {
        mon_resource.name.clone()
      }
    ));
    result.push(format!("  {}", types.join("/")));
  }

  Ok(result)
}

async fn print_abilities(args: &SubArgs) -> Result<Vec<String>, clap::error::Error> {
  let SubArgs::AbilityCmd {
    pokemon,
    fast,
    recursive,
    ..
  } = args
  else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create client
  let client = rustemon::client::RustemonClient::default();

  // Create pokemon resources
  let resources = match get_pokemon_from_chain(&client, &pokemon, *recursive).await {
    Ok(x) => x,
    Err(_) => panic!("error: could not find pokemon {}", pokemon),
  };

  // Create struct to store ability
  struct Ability {
    hidden: bool,
    ability: rustemon::model::pokemon::Ability,
  }

  let mut result = Vec::new();
  for mon_resource in resources.iter() {
    // Get abilities
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
      Err(_) => panic!("error: could not retrive abilities for pokemon {}", pokemon),
    };

    // Print English names
    let mut names = Vec::new();
    for ab in abilities.into_iter() {
      if *fast {
        names.push(ab.ability.name.clone() + if ab.hidden { " (hidden)" } else { "" });
      } else if let Ok(x) = get_name(&client, &ab.ability.names, "en").await {
        names.push(x + if ab.hidden { " (Hidden)" } else { "" });
      }
    }
    result.push(format!(
      "{}:",
      if !fast
        && let Ok(species) = mon_resource.species.follow(&client).await
        && let Ok(name) = get_name(&client, &species.names, "en").await
      {
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

async fn print_moves(args: &SubArgs) -> Result<Vec<String>, clap::error::Error> {
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

  // Create client
  let client = rustemon::client::RustemonClient::default();

  // Create pokemon resource
  let mon_resource = match pokemon::get_by_name(&pokemon, &client).await {
    Ok(x) => x,
    Err(_) => panic!("error: could not find pokemon {}", pokemon),
  };

  // Create struct to store move
  #[derive(Debug)]
  struct Move {
    name: String,
    level: i64,
  }

  // Get full learnset
  let mut moves = Vec::new();
  for move_resource in mon_resource.moves.iter() {
    for details in move_resource.version_group_details.iter() {
      if details.move_learn_method.name == "level-up"
        && details.version_group.name == format!("{}", *vgroup)
      {
        match *level {
          Some(x) if details.level_learned_at > x => {},
          _ => {
            if *fast {
              moves.push(Move {
                name: move_resource.move_.name.clone(),
                level: details.level_learned_at,
              });
            } else if let Ok(x) = move_resource.move_.follow(&client).await {
              if let Ok(y) = get_name(&client, &x.names, "en").await {
                moves.push(Move {
                  name: y,
                  level: details.level_learned_at,
                });
              }
            } else {
              panic!("error: could not find move {}", move_resource.move_.name);
            }
          },
        };
      }
    }
  }
  // Sort moves by descending level
  moves.sort_by(|m, n| n.level.cmp(&m.level));

  // Print result
  let mut moves = if let Some(_) = *level {
    moves.iter().take(4).collect::<Vec<_>>()
  } else {
    moves.iter().collect::<Vec<_>>()
  };
  moves.reverse();

  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast
      && let Ok(species) = mon_resource.species.follow(&client).await
      && let Ok(name) = get_name(&client, &species.names, "en").await
    {
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

async fn print_eggs(args: &SubArgs) -> Result<Vec<String>, clap::error::Error> {
  let SubArgs::EggCmd { pokemon, fast, .. } = args else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create client
  let client = rustemon::client::RustemonClient::default();

  // Create pokemon resources
  let species_resource = match pokemon_species::get_by_name(&pokemon, &client).await {
    Ok(x) => x,
    Err(_) => panic!("error: could not find pokemon species {}", pokemon),
  };

  // Get egg groups
  let eggs = match future::try_join_all(
    species_resource
      .egg_groups
      .iter()
      .map(async |g| g.follow(&client).await),
  )
  .await
  {
    Ok(x) => x,
    Err(_) => panic!(
      "error: could not retrive egg groups for pokemon {}",
      pokemon
    ),
  };

  // Print English names
  let eggs = if *fast {
    eggs.into_iter().map(|g| g.name).collect()
  } else {
    match future::try_join_all(
      eggs
        .into_iter()
        .map(|g| g.names)
        .map(async |names| get_name(&client, &names, "en").await),
    )
    .await
    {
      Ok(x) => x,
      Err(_) => panic!("error: could not find English names for egg groups"),
    }
  };

  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast && let Ok(name) = get_name(&client, &species_resource.names, "en").await {
      name
    } else {
      species_resource.name.clone()
    }
  ));
  eggs.iter().for_each(|x| result.push(format!(" - {}", x)));
  Ok(result)
}

async fn print_genders(args: &SubArgs) -> Result<Vec<String>, clap::error::Error> {
  let SubArgs::GenderCmd { pokemon, fast, .. } = args else {
    return Err(Args::command().error(ErrorKind::InvalidValue, "invalid arguments for subcommand"));
  };

  // Create client
  let client = rustemon::client::RustemonClient::default();

  // Create pokemon resources
  let species_resource = match pokemon_species::get_by_name(&pokemon, &client).await {
    Ok(x) => x,
    Err(_) => panic!("error: could not find pokemon species {}", pokemon),
  };

  let mut result = Vec::new();
  result.push(format!(
    "{}:",
    if !fast && let Ok(name) = get_name(&client, &species_resource.names, "en").await {
      name
    } else {
      species_resource.name.clone()
    }
  ));
  let rate = species_resource.gender_rate as f64 / 8.0 * 100.0;
  if rate < 0.0 {
    result.push(format!(" Genderless"));
  } else {
    result.push(format!(" M: {:>5.1}", 100.0 - rate));
    result.push(format!(" F: {:>5.1}", rate));
  }

  Ok(result)
}

async fn print_encounters(args: &SubArgs) -> Result<Vec<String>, clap::error::Error> {
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

  // Create client
  let client = rustemon::client::RustemonClient::default();

  // Create pokemon resources
  let resources = match get_pokemon_from_chain(&client, &pokemon, *recursive).await {
    Ok(x) => x,
    Err(_) => panic!("error: could not find pokemon {}", pokemon),
  };

  let mut result = Vec::new();
  for mon_resource in resources.iter() {
    let encounter_resources: Vec<rustemon::model::pokemon::LocationAreaEncounter> = if let Ok(mut x) =
      ureq::get(mon_resource.location_area_encounters.clone()).call()
      && let Ok(y) = x.body_mut().read_to_string()
    {
      serde_json::from_str(&y).expect("JSON was not well formatted")
    } else {
      panic!(
        "error: could not follow url for encounters for pokemon {}",
        pokemon
      );
    };

    let mut encounters = Vec::new();
    for enc in encounter_resources.iter() {
      for det in enc.version_details.iter() {
        if det.version.name == *version {
          encounters.push(if *fast {
            enc.location_area.name.clone()
          } else if let Ok(x) = enc.location_area.follow(&client).await
            && let Ok(y) = get_name(&client, &x.names, "en").await
          {
            y
          } else {
            panic!("error: could not find location area name");
          });
          break;
        }
      }
    }
    result.push(format!(
      "{}:",
      if !fast
        && let Ok(species) = mon_resource.species.follow(&client).await
        && let Ok(name) = get_name(&client, &species.names, "en").await
      {
        name
      } else {
        mon_resource.name.clone()
      }
    ));
    encounters
      .into_iter()
      .for_each(|x| result.push(format!(" - {}", x)));
  }

  Ok(result)
}
