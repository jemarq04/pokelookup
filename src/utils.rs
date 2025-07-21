use clap::{Parser, Subcommand, ValueEnum};

/// Look up pokemon details using PokeAPI using the 'rustemon' wrapper. Note that sometimes pokemon need to be listed
/// with their forms if the form is distinct enough (e.g. pumkaboo-small or toxtricity-amped). These varieties can be
/// listed using the 'list' subcommand.
#[derive(Parser, Debug)]
#[command(long_about)]
pub struct Args {
  #[command(subcommand)]
  pub command: SubArgs,
}

#[derive(Subcommand, Debug)]
pub enum SubArgs {
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
    #[arg(value_enum, hide_possible_values = true, help = "name of version")]
    version: Version,

    #[arg(help = "name of pokemon")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,

    #[arg(short, help = "recursively check evolution chain")]
    recursive: bool,
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
