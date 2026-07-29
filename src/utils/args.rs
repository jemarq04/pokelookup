use crate::utils::enums::*;
use clap::Args;

// ========================================================================================================================
// Subcommand Arguments
// ========================================================================================================================

#[derive(Args, Debug)]
pub struct ListArgs {
  #[arg(help = "name of pokemon species")]
  pub pokemon: String,

  #[arg(short, long, help = "skip API requests for formatted names")]
  pub fast: bool,

  #[arg(value_enum,
    short = 'L',
    long,
    value_name = "LANGUAGE",
    default_value_t = LanguageId::En,
    hide_possible_values=true,
    help = "language ID for API requests for formatted names"
  )]
  pub lang: LanguageId,
}

#[derive(Args, Debug)]
pub struct TypeArgs {
  #[arg(help = "name of pokemon")]
  pub pokemon: String,

  #[arg(short, long, help = "skip API requests for formatted names")]
  pub fast: bool,

  #[arg(short, long = "gen", help = "generation to query")]
  pub generation: Option<i64>,

  #[arg(value_enum,
    short = 'L',
    long,
    value_name = "LANGUAGE",
    default_value_t = LanguageId::En,
    hide_possible_values=true,
    help = "language ID for API requests for formatted names"
  )]
  pub lang: LanguageId,

  #[arg(short, help = "recursively check evolution chain")]
  pub recursive: bool,
}

#[derive(Args, Debug)]
pub struct AbilityArgs {
  #[arg(help = "name of pokemon")]
  pub pokemon: String,

  #[arg(short, long, help = "skip API requests for formatted names")]
  pub fast: bool,

  #[arg(value_enum,
    short = 'L',
    long,
    value_name = "LANGUAGE",
    default_value_t = LanguageId::En,
    hide_possible_values=true,
    help = "language ID for API requests for formatted names"
  )]
  pub lang: LanguageId,

  #[arg(short, help = "recursively check evolution chain")]
  pub recursive: bool,
}

#[derive(Args, Debug)]
pub struct MoveArgs {
  #[arg(help = "name of pokemon")]
  pub pokemon: String,

  #[arg(short, long, help = "skip API requests for formatted names")]
  pub fast: bool,

  #[arg(value_enum,
    short = 'L',
    long,
    value_name = "LANGUAGE",
    default_value_t = LanguageId::En,
    hide_possible_values=true,
    help = "language ID for API requests for formatted names"
  )]
  pub lang: LanguageId,

  #[arg(value_enum, short, long, default_value_t=VersionGroup::ScarletViolet,
          hide_possible_values=true, help="version group name")]
  pub vgroup: VersionGroup,

  #[arg(short, long, help = "request default moveset at given level")]
  pub level: Option<i64>,
}

#[derive(Args, Debug)]
pub struct EggArgs {
  #[arg(help = "name of pokemon species")]
  pub pokemon: String,

  #[arg(short, long, help = "skip API requests for formatted names")]
  pub fast: bool,

  #[arg(value_enum,
    short = 'L',
    long,
    value_name = "LANGUAGE",
    default_value_t = LanguageId::En,
    hide_possible_values=true,
    help = "language ID for API requests for formatted names"
  )]
  pub lang: LanguageId,
}

#[derive(Args, Debug)]
pub struct GenderArgs {
  #[arg(help = "name of pokemon species")]
  pub pokemon: String,

  #[arg(short, long, help = "skip API requests for formatted names")]
  pub fast: bool,

  #[arg(value_enum,
    short = 'L',
    long,
    value_name = "LANGUAGE",
    default_value_t = LanguageId::En,
    hide_possible_values=true,
    help = "language ID for API requests for formatted names"
  )]
  pub lang: LanguageId,
}

#[derive(Args, Debug)]
pub struct EncounterArgs {
  #[arg(value_enum, hide_possible_values = true, help = "name of version")]
  pub version: Version,

  #[arg(help = "name of pokemon")]
  pub pokemon: String,

  #[arg(short, long, help = "skip API requests for formatted names")]
  pub fast: bool,

  #[arg(value_enum,
    short = 'L',
    long,
    value_name = "LANGUAGE",
    default_value_t = LanguageId::En,
    hide_possible_values=true,
    help = "language ID for API requests for formatted names"
  )]
  pub lang: LanguageId,

  #[arg(short, help = "recursively check evolution chain")]
  pub recursive: bool,

  #[arg(short, long, help = "condense output by only showing location areas")]
  pub condensed: bool,
}

#[derive(Args, Debug)]
pub struct EvolutionArgs {
  #[arg(help = "name of pokemon species")]
  pub pokemon: String,

  #[arg(short, long, help = "skip API requests for formatted names")]
  pub fast: bool,

  #[arg(value_enum,
    short = 'L',
    long,
    value_name = "LANGUAGE",
    default_value_t = LanguageId::En,
    hide_possible_values=true,
    help = "language ID for API requests for formatted names"
  )]
  pub lang: LanguageId,

  #[arg(
    short,
    long,
    help = "hide the names of the pokemon in the evolution chain"
  )]
  pub secret: bool,

  #[arg(short, long, help = "show all evolution chains, even outdated ones")]
  pub all: bool,
}

#[derive(Args, Debug)]
pub struct MatchupArgs {
  #[arg(
    value_enum,
    hide_possible_values = true,
    value_name = "TYPE",
    help = "name of type"
  )]
  pub primary: Type,

  #[arg(
    value_enum,
    hide_possible_values = true,
    value_name = "TYPE",
    help = "name of optional secondary type"
  )]
  pub secondary: Option<Type>,

  #[arg(short, long, help = "print output as a list instead of a table")]
  pub list: bool,

  #[arg(short, long, help = "skip API requests for formatted names")]
  pub fast: bool,

  #[arg(value_enum,
    short = 'L',
    long,
    value_name = "LANGUAGE",
    default_value_t = LanguageId::En,
    hide_possible_values=true,
    help = "language ID for API requests for formatted names"
  )]
  pub lang: LanguageId,
}

#[derive(Args, Debug)]
pub struct WebArgs {
  #[command(flatten)]
  pub endpoint: Endpoints,

  #[arg(short = 'A', long, help = "name of area within region")]
  pub area: Option<String>,

  #[arg(short, long = "gen", help = "optional name of generation to use")]
  pub generation: Option<i64>,

  #[arg(short, long, help = "suppress print statements")]
  pub quiet: bool,
}

#[cfg(feature = "web")]
#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct Endpoints {
  #[arg(short, long, help_heading = "Endpoints", conflicts_with_all = ["area"], help = "name of pokemon")]
  pub pokemon: Option<String>,

  #[arg(short, long, help_heading = "Endpoints", help = "name of region")]
  pub region: Option<String>,

  #[arg(
    short,
    long,
    help_heading = "Endpoints",
    conflicts_with = "area",
    help = "name of move"
  )]
  pub move_: Option<String>,

  #[arg(short, long, help_heading = "Endpoints", conflicts_with_all = ["area", "generation"], help = "name of ability")]
  pub ability: Option<String>,

  #[arg(short, long, help_heading = "Endpoints", conflicts_with_all = ["area", "generation"], help = "name of item")]
  pub item: Option<String>,
}

#[cfg(feature = "web")]
impl Endpoints {
  pub fn get_mode(&self) -> DexMode {
    if let Some(name) = &self.pokemon {
      DexMode::Pokedex(name.clone())
    } else if let Some(name) = &self.region {
      DexMode::Pokearth(name.clone())
    } else if let Some(name) = &self.move_ {
      DexMode::Attackdex(name.clone())
    } else if let Some(name) = &self.ability {
      DexMode::Abilitydex(name.clone())
    } else if let Some(name) = &self.item {
      DexMode::Itemdex(name.clone())
    } else {
      unreachable!()
    }
  }
}

#[cfg(feature = "web")]
pub enum DexMode {
  Pokedex(String),
  Pokearth(String),
  Attackdex(String),
  Abilitydex(String),
  Itemdex(String),
}
