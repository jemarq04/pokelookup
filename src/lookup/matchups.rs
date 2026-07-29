use crate::get_name;
use crate::utils::args::MatchupArgs;
use crate::utils::cli;
use clap::error::ErrorKind;
use itertools::izip;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::type_;

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

  // Get matchups from other types
  let mut no_damage_from = Vec::new();
  let mut half_damage_from = Vec::new();
  let mut double_damage_from = Vec::new();
  let mut quarter_damage_from = Vec::new();
  let mut quad_damage_from = Vec::new();

  for other_type in &primary.damage_relations.no_damage_from {
    no_damage_from.push(if args.fast {
      other_type.name.clone()
    } else {
      get_name!(follow other_type, client, args.lang.to_string())
    });
  }
  for other_type in &primary.damage_relations.half_damage_from {
    half_damage_from.push(if args.fast {
      other_type.name.clone()
    } else {
      get_name!(follow other_type, client, args.lang.to_string())
    });
  }
  for other_type in &primary.damage_relations.double_damage_from {
    double_damage_from.push(if args.fast {
      other_type.name.clone()
    } else {
      get_name!(follow other_type, client, args.lang.to_string())
    });
  }
  if let Some(ref second) = secondary {
    for other_type in &second.damage_relations.no_damage_from {
      let name = if args.fast {
        other_type.name.clone()
      } else {
        get_name!(follow other_type, client, args.lang.to_string())
      };
      if let Some(idx) = half_damage_from.iter().position(|x| *x == name) {
        half_damage_from.remove(idx);
        no_damage_from.push(name.clone());
      } else if let Some(idx) = double_damage_from.iter().position(|x| *x == name) {
        double_damage_from.remove(idx);
        no_damage_from.push(name.clone());
      } else if !no_damage_from.contains(&name) {
        no_damage_from.push(name.clone());
      }
    }
    for other_type in &second.damage_relations.half_damage_from {
      let name = if args.fast {
        other_type.name.clone()
      } else {
        get_name!(follow other_type, client, args.lang.to_string())
      };
      if let Some(idx) = half_damage_from.iter().position(|x| *x == name) {
        quarter_damage_from.push(name.clone());
        half_damage_from.remove(idx);
      } else if let Some(idx) = double_damage_from.iter().position(|x| *x == name) {
        double_damage_from.remove(idx);
      } else if !no_damage_from.contains(&name) {
        half_damage_from.push(name.clone());
      }
    }
    for other_type in &second.damage_relations.double_damage_from {
      let name = if args.fast {
        other_type.name.clone()
      } else {
        get_name!(follow other_type, client, args.lang.to_string())
      };
      if let Some(idx) = half_damage_from.iter().position(|x| *x == name) {
        half_damage_from.remove(idx);
      } else if let Some(idx) = double_damage_from.iter().position(|x| *x == name) {
        quad_damage_from.push(name.clone());
        double_damage_from.remove(idx);
      } else if !no_damage_from.contains(&name) {
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
        if no_damage_from.iter().filter(|x| !x.is_empty()).count() != 0 {
          add_separator = true;
          result.push(String::from(" - 0x:"));
          for no_dmg in &no_damage_from {
            if no_dmg.is_empty() {
              break;
            }
            result.push(format!("   * {no_dmg}"));
          }
        }
        if half_damage_from.iter().filter(|x| !x.is_empty()).count() != 0 {
          if add_separator {
            result.push(String::new());
          }
          add_separator = true;
          result.push(String::from(" - 0.5x:"));
          for half_dmg in &half_damage_from {
            if half_dmg.is_empty() {
              break;
            }
            result.push(format!("   * {half_dmg}"));
          }
        }
        if double_damage_from.iter().filter(|x| !x.is_empty()).count() != 0 {
          if add_separator {
            result.push(String::new());
          }
          result.push(String::from(" - 2x:"));
          for double_dmg in &double_damage_from {
            if double_dmg.is_empty() {
              break;
            }
            result.push(format!("   * {double_dmg}"));
          }
        }
      } else {
        result.push(format!("{:^12} {:^12} {:^12}", "*0", "*0.5", "*2"));
        result.push(format!("{:-<12} {:-<12} {:-<12}", "", "", ""));
        for (no_dmg, half_dmg, double_dmg) in
          izip!(&no_damage_from, &half_damage_from, &double_damage_from)
        {
          result.push(format!("{no_dmg:<12} {half_dmg:<12} {double_dmg:<12}"));
        }
      }
    },
    Some(second) => {
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
              second.name.clone()
            } else {
              get_name!(second, client, args.lang.to_string())
            }
          ]
          .join("/"),
        ));
        let mut add_separator = false;
        if no_damage_from.iter().filter(|x| !x.is_empty()).count() != 0 {
          add_separator = true;
          result.push(String::from(" - 0x:"));
          for no_dmg in &no_damage_from {
            if no_dmg.is_empty() {
              break;
            }
            result.push(format!("   * {no_dmg}"));
          }
        }
        if quarter_damage_from.iter().filter(|x| !x.is_empty()).count() != 0 {
          if add_separator {
            result.push(String::new());
          }
          add_separator = true;
          result.push(String::from(" - 0.25x:"));
          for quarter_dmg in &quarter_damage_from {
            if quarter_dmg.is_empty() {
              break;
            }
            result.push(format!("   * {quarter_dmg}"));
          }
        }
        if half_damage_from.iter().filter(|x| !x.is_empty()).count() != 0 {
          if add_separator {
            result.push(String::new());
          }
          add_separator = true;
          result.push(String::from(" - 0.5x:"));
          for half_dmg in &half_damage_from {
            if half_dmg.is_empty() {
              break;
            }
            result.push(format!("   * {half_dmg}"));
          }
        }
        if double_damage_from.iter().filter(|x| !x.is_empty()).count() != 0 {
          if add_separator {
            result.push(String::new());
          }
          add_separator = true;
          result.push(String::from(" - 2x:"));
          for double_dmg in &double_damage_from {
            if double_dmg.is_empty() {
              break;
            }
            result.push(format!("   * {double_dmg}"));
          }
        }
        if quad_damage_from.iter().filter(|x| !x.is_empty()).count() != 0 {
          if add_separator {
            result.push(String::new());
          }
          result.push(String::from(" - 4x:"));
          for quad_dmg in &quad_damage_from {
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
          &no_damage_from, &quarter_damage_from, &half_damage_from, &double_damage_from,
          &quad_damage_from
        ) {
          result.push(format!(
            "{no_dmg:<12} {quarter_dmg:<12} {half_dmg:<12} {double_dmg:<12} {quad_dmg:<12}"
          ));
        }
      }
    },
  }

  Ok(result)
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
