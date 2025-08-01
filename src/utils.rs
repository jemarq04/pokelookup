use clap::{Parser, Subcommand, ValueEnum};
use futures::future;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::*;

/// Look up pokemon details using PokeAPI using the 'rustemon' wrapper. Note that sometimes pokemon need to be listed
/// with their forms if the form is distinct enough (e.g. pumkaboo-small or toxtricity-amped). These varieties can be
/// listed using the 'list' subcommand.
#[derive(Parser, Debug)]
#[command(version, long_about)]
pub struct Args {
  #[arg(
    long,
    value_name = "DIR",
    help = "cache directory for API calls (default: ~/.cache/pokelookup/)"
  )]
  pub cache_dir: Option<std::path::PathBuf>,

  #[command(subcommand)]
  pub command: SubArgs,
}

#[derive(Subcommand, Debug)]
pub enum SubArgs {
  /// Look up the varieties of a given pokemon.
  #[command(name = "list", long_about)]
  ListCmd {
    #[arg(help = "name of pokemon species")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,
  },

  /// Look up the type(s) of a given pokemon.
  #[command(name = "types", long_about)]
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
    about = "Look up the abilities of a given pokemon",
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
    about = "Look up the level-up moveset of a given pokemon",
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
  #[command(name = "eggs", long_about)]
  EggCmd {
    #[arg(help = "name of pokemon species")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,
  },

  /// Look up the gender ratio of a given pokemon species.
  #[command(name = "genders", long_about)]
  GenderCmd {
    #[arg(help = "name of pokemon species")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,
  },

  /// Look up the encounters for a given pokemon and version.
  #[command(name = "encounters", long_about)]
  EncounterCmd {
    #[arg(value_enum, hide_possible_values = true, help = "name of version")]
    version: Version,

    #[arg(help = "name of pokemon")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,

    #[arg(short, help = "recursively check evolution chain")]
    recursive: bool,
  },

  /// Look up the type weaknesses/resistances for given type(s).
  #[command(name = "matchups", long_about)]
  MatchupCmd {
    #[arg(
      value_enum,
      hide_possible_values = true,
      value_name = "TYPE",
      help = "name of type"
    )]
    primary: Type,

    #[arg(
      value_enum,
      hide_possible_values = true,
      value_name = "TYPE",
      help = "name of optional secondary type"
    )]
    secondary: Option<Type>,
  },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum VersionGroup {
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Version {
  Red,
  Blue,
  Yellow,
  Gold,
  Silver,
  Crystal,
  Ruby,
  Sapphire,
  Emerald,
  Firered,
  Leafgreen,
  Diamond,
  Pearl,
  Platinum,
  Heartgold,
  Soulsilver,
  Black,
  White,
  Colosseum,
  #[value(alias = "xd")]
  XD,
  Black2,
  White2,
  X,
  Y,
  OmegaRuby,
  AlphaSapphire,
  Sun,
  Moon,
  UltraSun,
  UltraMoon,
  LetsGoPikachu,
  LetsGoEevee,
  Sword,
  Shield,
  TheIsleOfArmor,
  TheCrownTundra,
  BrilliantDiamond,
  ShiningPearl,
  LegendsArceus,
  Scarlet,
  Violet,
  TheTealMask,
  TheIndigoDisk,
}
impl std::fmt::Display for Version {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self
      .to_possible_value()
      .expect("no values are skipped")
      .get_name()
      .fmt(f)
  }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Type {
  Normal,
  Fighting,
  Flying,
  Poison,
  Ground,
  Rock,
  Bug,
  Ghost,
  Steel,
  Fire,
  Water,
  Grass,
  Electric,
  Psychic,
  Ice,
  Dragon,
  Dark,
  Fairy,
}
impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self
      .to_possible_value()
      .expect("no values are skipped")
      .get_name()
      .fmt(f)
  }
}

pub async fn get_name(
  client: &RustemonClient,
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

pub async fn get_pokemon_name(
  client: &RustemonClient,
  pokemon: &rustemon::model::pokemon::Pokemon,
  lang: &str,
) -> Result<String, ()> {
  let forms =
    match future::try_join_all(pokemon.forms.iter().map(async |f| f.follow(&client).await)).await {
      Ok(x) => x,
      Err(_) => return Err(()),
    };

  for form in forms.into_iter() {
    if !form.is_default || form.names.len() == 0 {
      continue;
    }
    return get_name(&client, &form.names, lang).await;
  }

  let species = match pokemon.species.follow(&client).await {
    Ok(x) => x,
    Err(_) => return Err(()),
  };

  get_name(&client, &species.names, lang).await
}

pub async fn get_pokemon_from_chain(
  client: &RustemonClient,
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
      if let Ok(x) = pokemon_species::get_by_name(&chain.species.name, &client).await {
        if let Ok(y) = future::try_join_all(
          x.varieties
            .iter()
            .map(async |v| v.pokemon.follow(&client).await),
        )
        .await
        {
          y.into_iter().for_each(|mon| result.push(mon));
        }
      }
      for evo1 in chain.evolves_to.iter() {
        if let Ok(x) = pokemon_species::get_by_name(&evo1.species.name, &client).await {
          if let Ok(y) = future::try_join_all(
            x.varieties
              .iter()
              .map(async |v| v.pokemon.follow(&client).await),
          )
          .await
          {
            y.into_iter().for_each(|mon| result.push(mon));
          }
        }
        for evo2 in evo1.evolves_to.iter() {
          if let Ok(x) = pokemon_species::get_by_name(&evo2.species.name, &client).await {
            if let Ok(y) = future::try_join_all(
              x.varieties
                .iter()
                .map(async |v| v.pokemon.follow(&client).await),
            )
            .await
            {
              y.into_iter().for_each(|mon| result.push(mon));
            }
          }
        }
      }
    }
  } else {
    result.push(pokemon);
  }

  Ok(result)
}

pub fn follow_encounters(
  pokemon: &rustemon::model::pokemon::Pokemon,
) -> Result<Vec<rustemon::model::pokemon::LocationAreaEncounter>, ()> {
  if let Ok(mut url) = ureq::get(pokemon.location_area_encounters.clone()).call()
    && let Ok(body) = url.body_mut().read_to_string()
  {
    let result: Vec<rustemon::model::pokemon::LocationAreaEncounter> =
      match serde_json::from_str(&body) {
        Ok(x) => x,
        Err(_) => {
          return Err(());
        },
      };
    return Ok(result);
  }
  Err(())
}
