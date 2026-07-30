use crate::get_name;
use crate::utils::args::MatchupArgs;
use crate::utils::cli;
use crate::utils::enums::LanguageId;
use clap::error::ErrorKind;
use itertools::izip;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::model::pokemon::Type;
use rustemon::pokemon::type_;

#[allow(clippy::struct_field_names)]
struct Matchups {
  no_damage_from: Vec<String>,
  half_damage_from: Vec<String>,
  double_damage_from: Vec<String>,
  quarter_damage_from: Vec<String>,
  quad_damage_from: Vec<String>,
}

async fn load_matchups(
  client: &RustemonClient,
  primary: &Type,
  secondary: Option<Type>,
  fast: bool,
  lang: LanguageId,
) -> Matchups {
  let mut result = Matchups {
    no_damage_from: Vec::new(),
    half_damage_from: Vec::new(),
    double_damage_from: Vec::new(),
    quarter_damage_from: Vec::new(),
    quad_damage_from: Vec::new(),
  };
  for other_type in &primary.damage_relations.no_damage_from {
    result.no_damage_from.push(if fast {
      other_type.name.clone()
    } else {
      get_name!(follow other_type, client, lang.to_string())
    });
  }
  for other_type in &primary.damage_relations.half_damage_from {
    result.half_damage_from.push(if fast {
      other_type.name.clone()
    } else {
      get_name!(follow other_type, client, lang.to_string())
    });
  }
  for other_type in &primary.damage_relations.double_damage_from {
    result.double_damage_from.push(if fast {
      other_type.name.clone()
    } else {
      get_name!(follow other_type, client, lang.to_string())
    });
  }
  if let Some(secondary) = secondary {
    for other_type in &secondary.damage_relations.no_damage_from {
      let name = if fast {
        other_type.name.clone()
      } else {
        get_name!(follow other_type, client, lang.to_string())
      };
      if let Some(idx) = result.half_damage_from.iter().position(|x| *x == name) {
        result.half_damage_from.remove(idx);
        result.no_damage_from.push(name.clone());
      } else if let Some(idx) = result.double_damage_from.iter().position(|x| *x == name) {
        result.double_damage_from.remove(idx);
        result.no_damage_from.push(name.clone());
      } else if !result.no_damage_from.contains(&name) {
        result.no_damage_from.push(name.clone());
      }
    }
    for other_type in &secondary.damage_relations.half_damage_from {
      let name = if fast {
        other_type.name.clone()
      } else {
        get_name!(follow other_type, client, lang.to_string())
      };
      if let Some(idx) = result.half_damage_from.iter().position(|x| *x == name) {
        result.quarter_damage_from.push(name.clone());
        result.half_damage_from.remove(idx);
      } else if let Some(idx) = result.double_damage_from.iter().position(|x| *x == name) {
        result.double_damage_from.remove(idx);
      } else if !result.no_damage_from.contains(&name) {
        result.half_damage_from.push(name.clone());
      }
    }
    for other_type in &secondary.damage_relations.double_damage_from {
      let name = if fast {
        other_type.name.clone()
      } else {
        get_name!(follow other_type, client, lang.to_string())
      };
      if let Some(idx) = result.half_damage_from.iter().position(|x| *x == name) {
        result.half_damage_from.remove(idx);
      } else if let Some(idx) = result.double_damage_from.iter().position(|x| *x == name) {
        result.quad_damage_from.push(name.clone());
        result.double_damage_from.remove(idx);
      } else if !result.no_damage_from.contains(&name) {
        result.double_damage_from.push(name.clone());
      }
    }
  }

  // Bring all vectors to the same size
  let maxlen = itertools::max(vec![
    result.no_damage_from.len(),
    result.half_damage_from.len(),
    result.double_damage_from.len(),
  ])
  .unwrap();
  while result.no_damage_from.len() < maxlen {
    result.no_damage_from.push(String::new());
  }
  while result.half_damage_from.len() < maxlen {
    result.half_damage_from.push(String::new());
  }
  while result.double_damage_from.len() < maxlen {
    result.double_damage_from.push(String::new());
  }
  while result.quarter_damage_from.len() < maxlen {
    result.quarter_damage_from.push(String::new());
  }
  while result.quad_damage_from.len() < maxlen {
    result.quad_damage_from.push(String::new());
  }

  result
}

async fn get_single_type_output(
  client: &RustemonClient,
  primary: &Type,
  args: MatchupArgs,
) -> Vec<String> {
  // Get matchups from other types
  let matchups = load_matchups(client, primary, None, args.fast, args.lang).await;

  // Return type matchups
  let mut result = Vec::new();
  if args.list {
    result.push(format!(
      "{}:",
      if args.fast {
        primary.name.clone()
      } else {
        get_name!(primary, client, args.lang.to_string())
      },
    ));
    let mut add_separator = false;
    if matchups
      .no_damage_from
      .iter()
      .filter(|x| !x.is_empty())
      .count()
      != 0
    {
      add_separator = true;
      result.push(String::from(" - 0x:"));
      for no_dmg in &matchups.no_damage_from {
        if no_dmg.is_empty() {
          break;
        }
        result.push(format!("   * {no_dmg}"));
      }
    }
    if matchups
      .half_damage_from
      .iter()
      .filter(|x| !x.is_empty())
      .count()
      != 0
    {
      if add_separator {
        result.push(String::new());
      }
      add_separator = true;
      result.push(String::from(" - 0.5x:"));
      for half_dmg in &matchups.half_damage_from {
        if half_dmg.is_empty() {
          break;
        }
        result.push(format!("   * {half_dmg}"));
      }
    }
    if matchups
      .double_damage_from
      .iter()
      .filter(|x| !x.is_empty())
      .count()
      != 0
    {
      if add_separator {
        result.push(String::new());
      }
      result.push(String::from(" - 2x:"));
      for double_dmg in &matchups.double_damage_from {
        if double_dmg.is_empty() {
          break;
        }
        result.push(format!("   * {double_dmg}"));
      }
    }
  } else {
    result.push(format!("{:^12} {:^12} {:^12}", "*0", "*0.5", "*2"));
    result.push(format!("{:-<12} {:-<12} {:-<12}", "", "", ""));
    for (no_dmg, half_dmg, double_dmg) in izip!(
      &matchups.no_damage_from, &matchups.half_damage_from, &matchups.double_damage_from
    ) {
      result.push(format!("{no_dmg:<12} {half_dmg:<12} {double_dmg:<12}"));
    }
  }
  result
}

async fn get_multi_type_output(
  client: &RustemonClient,
  primary: &Type,
  secondary: &Type,
  args: MatchupArgs,
) -> Vec<String> {
  // Get matchups from other types
  let matchups = load_matchups(
    client,
    primary,
    Some(secondary.clone()),
    args.fast,
    args.lang,
  )
  .await;

  // Return type matchups
  let mut result = Vec::new();
  if args.list {
    result.push(format!(
      "{}:",
      [
        if args.fast {
          primary.name.clone()
        } else {
          get_name!(primary, client, args.lang.to_string())
        },
        if args.fast {
          secondary.name.clone()
        } else {
          get_name!(secondary, client, args.lang.to_string())
        }
      ]
      .join("/"),
    ));
    let mut add_separator = false;
    if matchups
      .no_damage_from
      .iter()
      .filter(|x| !x.is_empty())
      .count()
      != 0
    {
      add_separator = true;
      result.push(String::from(" - 0x:"));
      for no_dmg in &matchups.no_damage_from {
        if no_dmg.is_empty() {
          break;
        }
        result.push(format!("   * {no_dmg}"));
      }
    }
    if matchups
      .quarter_damage_from
      .iter()
      .filter(|x| !x.is_empty())
      .count()
      != 0
    {
      if add_separator {
        result.push(String::new());
      }
      add_separator = true;
      result.push(String::from(" - 0.25x:"));
      for quarter_dmg in &matchups.quarter_damage_from {
        if quarter_dmg.is_empty() {
          break;
        }
        result.push(format!("   * {quarter_dmg}"));
      }
    }
    if matchups
      .half_damage_from
      .iter()
      .filter(|x| !x.is_empty())
      .count()
      != 0
    {
      if add_separator {
        result.push(String::new());
      }
      add_separator = true;
      result.push(String::from(" - 0.5x:"));
      for half_dmg in &matchups.half_damage_from {
        if half_dmg.is_empty() {
          break;
        }
        result.push(format!("   * {half_dmg}"));
      }
    }
    if matchups
      .double_damage_from
      .iter()
      .filter(|x| !x.is_empty())
      .count()
      != 0
    {
      if add_separator {
        result.push(String::new());
      }
      add_separator = true;
      result.push(String::from(" - 2x:"));
      for double_dmg in &matchups.double_damage_from {
        if double_dmg.is_empty() {
          break;
        }
        result.push(format!("   * {double_dmg}"));
      }
    }
    if matchups
      .quad_damage_from
      .iter()
      .filter(|x| !x.is_empty())
      .count()
      != 0
    {
      if add_separator {
        result.push(String::new());
      }
      result.push(String::from(" - 4x:"));
      for quad_dmg in &matchups.quad_damage_from {
        if quad_dmg.is_empty() {
          break;
        }
        result.push(format!("   * {quad_dmg}"));
      }
    }
  } else {
    result.push(format!(
      "{:^12} {:^12} {:^12} {:^12} {:^12}",
      "*0", "*0.25", "*0.5", "*2", "*4"
    ));
    result.push(format!(
      "{:-<12} {:-<12} {:-<12} {:-<12} {:-<12}",
      "", "", "", "", ""
    ));
    for (no_dmg, quarter_dmg, half_dmg, double_dmg, quad_dmg) in izip!(
      &matchups.no_damage_from, &matchups.quarter_damage_from, &matchups.half_damage_from,
      &matchups.double_damage_from, &matchups.quad_damage_from
    ) {
      result.push(format!(
        "{no_dmg:<12} {quarter_dmg:<12} {half_dmg:<12} {double_dmg:<12} {quad_dmg:<12}"
      ));
    }
  }
  result
}

pub async fn print_matchups(
  client: &RustemonClient,
  args: MatchupArgs,
) -> Result<Vec<String>, clap::Error> {
  // Get type resources
  let Ok(primary) = type_::get_by_name(&args.primary.to_string(), client).await else {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!("API error: could not retrieve type {}", args.primary),
    ));
  };
  let secondary = match args.secondary {
    Some(t) => match type_::get_by_name(&t.to_string(), client).await {
      Ok(x) => Some(x),
      Err(_) => {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!("API error: could not retrieve type {t}"),
        ));
      },
    },
    None => None,
  };

  Ok(match secondary {
    Some(secondary) => get_multi_type_output(client, &primary, &secondary, args).await,
    None => get_single_type_output(client, &primary, args).await,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::enums::{LanguageId, Type};

  #[tokio::test]
  async fn test_matchups() {
    let client = RustemonClient::default();

    let success = vec![
      "     *0          *0.5          *2     ",
      "------------ ------------ ------------",
      "Dragon       Fighting     Poison      ",
      "             Bug          Steel       ",
      "             Dark                     ",
    ];

    let args = MatchupArgs {
      primary: Type::Fairy,
      secondary: None,
      list: false,
      fast: false,
      lang: LanguageId::En,
    };

    match print_matchups(&client, args).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }

  #[tokio::test]
  async fn test_matchups_dual() {
    let client = RustemonClient::default();

    let success = vec![
      "     *0         *0.25         *0.5          *2           *4     ",
      "------------ ------------ ------------ ------------ ------------",
      "Electric                  Flying       Ground                   ",
      "                          Steel        Water                    ",
      "                          Poison       Grass                    ",
      "                          Rock         Ice                      ",
    ];

    let args = MatchupArgs {
      primary: Type::Electric,
      secondary: Some(Type::Ground),
      list: false,
      fast: false,
      lang: LanguageId::En,
    };

    match print_matchups(&client, args).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }

  #[tokio::test]
  async fn test_matchups_list() {
    let client = RustemonClient::default();

    let success = vec![
      "Hada/Acero:", " - 0x:", "   * Dragón", "   * Veneno", "", " - 0.25x:", "   * Bicho", "",
      " - 0.5x:", "   * Siniestro", "   * Normal", "   * Volador", "   * Roca", "   * Planta",
      "   * Psíquico", "   * Hielo", "   * Hada", "", " - 2x:", "   * Tierra", "   * Fuego",
    ];

    let args = MatchupArgs {
      primary: Type::Fairy,
      secondary: Some(Type::Steel),
      list: true,
      fast: false,
      lang: LanguageId::Es,
    };

    match print_matchups(&client, args).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }
}
