use crate::impl_Display;
use clap::builder::styling::{AnsiColor, Effects, Style, Styles};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};

pub const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
pub const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
pub const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

/// Cargo's color style
/// [source](https://github.com/crate-ci/clap-cargo/blob/master/src/style.rs)
const CARGO_STYLING: Styles = Styles::styled()
  .header(HEADER)
  .usage(USAGE)
  .literal(LITERAL)
  .placeholder(PLACEHOLDER)
  .error(ERROR)
  .valid(VALID)
  .invalid(INVALID);

/// Look up pokemon details using PokeAPI using the 'rustemon' wrapper. Note that sometimes pokemon need to be listed
/// with their forms if the form is distinct enough (e.g. pumkaboo-small or toxtricity-amped). These varieties can be
/// listed using the 'list' subcommand.
#[derive(Parser, Debug)]
#[command(version, long_about, styles=CARGO_STYLING)]
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

    #[arg(value_enum,
      short = 'L',
      long,
      value_name = "LANGUAGE",
      default_value_t = LanguageId::En,
      hide_possible_values=true,
      help = "language ID for API requests for formatted names"
    )]
    lang: LanguageId,
  },

  /// Look up the type(s) of a given pokemon.
  #[command(name = "types", long_about)]
  TypeCmd {
    #[arg(help = "name of pokemon")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,

    #[arg(value_enum,
      short = 'L',
      long,
      value_name = "LANGUAGE",
      default_value_t = LanguageId::En,
      hide_possible_values=true,
      help = "language ID for API requests for formatted names"
    )]
    lang: LanguageId,

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

    #[arg(value_enum,
      short = 'L',
      long,
      value_name = "LANGUAGE",
      default_value_t = LanguageId::En,
      hide_possible_values=true,
      help = "language ID for API requests for formatted names"
    )]
    lang: LanguageId,

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

    #[arg(value_enum,
      short = 'L',
      long,
      value_name = "LANGUAGE",
      default_value_t = LanguageId::En,
      hide_possible_values=true,
      help = "language ID for API requests for formatted names"
    )]
    lang: LanguageId,

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

    #[arg(value_enum,
      short = 'L',
      long,
      value_name = "LANGUAGE",
      default_value_t = LanguageId::En,
      hide_possible_values=true,
      help = "language ID for API requests for formatted names"
    )]
    lang: LanguageId,
  },

  /// Look up the gender ratio of a given pokemon species.
  #[command(name = "genders", long_about)]
  GenderCmd {
    #[arg(help = "name of pokemon species")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,

    #[arg(value_enum,
      short = 'L',
      long,
      value_name = "LANGUAGE",
      default_value_t = LanguageId::En,
      hide_possible_values=true,
      help = "language ID for API requests for formatted names"
    )]
    lang: LanguageId,
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

    #[arg(value_enum,
      short = 'L',
      long,
      value_name = "LANGUAGE",
      default_value_t = LanguageId::En,
      hide_possible_values=true,
      help = "language ID for API requests for formatted names"
    )]
    lang: LanguageId,

    #[arg(short, help = "recursively check evolution chain")]
    recursive: bool,
  },

  /// Look up evolution chain for a given pokemon species.
  #[command(name = "evolutions", long_about)]
  EvolutionCmd {
    #[arg(help = "name of pokemon species")]
    pokemon: String,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,

    #[arg(value_enum,
      short = 'L',
      long,
      value_name = "LANGUAGE",
      default_value_t = LanguageId::En,
      hide_possible_values=true,
      help = "language ID for API requests for formatted names"
    )]
    lang: LanguageId,

    #[arg(
      short,
      long,
      help = "hide the names of the pokemon in the evolution chain"
    )]
    secret: bool,

    #[arg(short, long, help = "show all evolution chains, even outdated ones")]
    all: bool,
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

    #[arg(short, long, help = "print output as a list instead of a table")]
    list: bool,

    #[arg(short, long, help = "skip API requests for formatted names")]
    fast: bool,

    #[arg(value_enum,
      short = 'L',
      long,
      value_name = "LANGUAGE",
      default_value_t = LanguageId::En,
      hide_possible_values=true,
      help = "language ID for API requests for formatted names"
    )]
    lang: LanguageId,
  },

  /// Open web pages for a given endpoint. A valid endpoint includes pokemon, abilities, items, and more.
  #[cfg(feature = "web")]
  #[command(name = "dex", long_about)]
  DexCmd {

    #[command(flatten)]
    endpoint: Endpoints,

    #[arg(short = 'A', long, help = "name of area within region")]
    area: Option<String>,

    #[arg(short, long = "gen", help = "optional name of generation to use")]
    generation: Option<i64>,
  },
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct Endpoints {
    #[arg(short, long, conflicts_with_all = ["area"], help = "name of pokemon")]
    pokemon: Option<String>,

    #[arg(short, long, help = "name of region")]
    region: Option<String>,

    #[arg(short, long, conflicts_with = "area", help = "name of move")]
    move_: Option<String>,

    #[arg(short, long, conflicts_with_all = ["area", "generation"], help = "name of ability")]
    ability: Option<String>,

    #[arg(short, long, conflicts_with_all = ["area", "generation"], help = "name of item")]
    item: Option<String>,
}

pub fn get_appname() -> String {
  String::from(Args::command().get_name())
}

pub fn error(kind: clap::error::ErrorKind, message: String) -> clap::Error {
  Args::command().error(kind, message)
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
impl_Display!(VersionGroup);

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
impl_Display!(Version);

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
impl_Display!(Type);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum LanguageId {
  #[value(alias = "ja-Hrkt")]
  JaHrkt,
  Roomaji,
  Ko,
  #[value(alias = "zh-Hant")]
  ZhHant,
  Fr,
  De,
  Es,
  It,
  En,
  Cs,
  Ja,
  #[value(alias = "zh-Hans")]
  ZhHans,
  #[value(alias = "pt-BR")]
  PtBR,
}
impl_Display!(LanguageId);
