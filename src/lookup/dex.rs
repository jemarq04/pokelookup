use super::pokedex::Pokedex;
use crate::svec;
use crate::utils::cli;
use clap::ValueEnum;
use clap::error::ErrorKind;

const LATEST_GEN: i64 = 9;

pub fn open_pokedex(pokemon: String, generation: Option<i64>) -> Result<Vec<String>, clap::Error> {
  let pokemon = pokemon.to_lowercase().replace(" ", "");
  match generation {
    None | Some(0) => {
      if let Err(_) = open::that(format!("https://www.serebii.net/pokemon/{pokemon}")) {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!("couldn't open page for {pokemon}"),
        ));
      }
    },
    Some(g) => match g {
      g @ 8..=LATEST_GEN => {
        if let Err(_) = open::that(format!(
          "https://www.serebii.net/pokedex-{}/{pokemon}",
          match g {
            8 => "swsh",
            9 => "sv",
            _ => unreachable!(),
          }
        )) {
          return Err(cli::error(
            ErrorKind::InvalidValue,
            format!("couldn't open page for {pokemon}"),
          ));
        }
      },
      g @ 1..8 => {
        let num = match Pokedex::from_str(&pokemon, true) {
          Ok(n) => n as i64,
          Err(_) => {
            return Err(cli::error(
              ErrorKind::InvalidValue,
              format!("invalid pokemon species: {pokemon}"),
            ));
          },
        };
        if let Err(_) = open::that(format!(
          "https://www.serebii.net/pokedex{}/{num:0>3}.shtml",
          match g {
            1 => "",
            2 => "-gs",
            3 => "-rs",
            4 => "-dp",
            5 => "-bw",
            6 => "-xy",
            7 => "-sm",
            _ => unreachable!(),
          },
        )) {
          return Err(cli::error(
            ErrorKind::InvalidValue,
            format!("couldn't open page for {pokemon}"),
          ));
        }
      },
      _ => {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!("invalid generation: {g}"),
        ));
      },
    },
  }
  Ok(svec!["Opened page successfully."])
}

pub fn open_pokearth(
  region: String,
  area: Option<String>,
  generation: Option<i64>,
) -> Result<Vec<String>, clap::Error> {
  let region = region.to_lowercase();
  let area = match area {
    Some(x) => Some(x.to_lowercase().replace(" ", "")),
    None => None,
  };

  let mut url = format!("https://www.serebii.net/pokearth/{region}/");
  if let Some(area) = &area {
    url += &format!(
      "{}{}.shtml",
      match generation {
        None | Some(0) => String::new(),
        Some(g) => match g {
          1 => String::from("1st/"),
          2 => String::from("2nd/"),
          3 => String::from("3rd/"),
          g @ 4..=LATEST_GEN => format!("{g}th/"),
          _ => {
            return Err(cli::error(
              ErrorKind::InvalidValue,
              format!("invalid generation: {g}"),
            ));
          },
        },
      },
      area,
    );
  }

  if let Err(_) = open::that(url) {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!(
        "couldn't open page for {}",
        match area {
          Some(area) => area,
          None => region,
        },
      ),
    ));
  }

  Ok(svec!["Opened page successfully."])
}

pub fn open_attackdex(move_: String, generation: Option<i64>) -> Result<Vec<String>, clap::Error> {
  fn get_genstr(num: i64) -> Result<String, clap::Error> {
    match num {
      1 => Ok(String::from("-rby")),
      2 => Ok(String::from("-gs")),
      3 => Ok(String::new()),
      4 => Ok(String::from("-dp")),
      5 => Ok(String::from("-bw")),
      6 => Ok(String::from("-xy")),
      7 => Ok(String::from("-sm")),
      8 => Ok(String::from("-swsh")),
      9 => Ok(String::from("-sv")),
      _ => Err(cli::error(
        ErrorKind::InvalidValue,
        format!("invalid generation: {num}"),
      )),
    }
  }
  let move_ = move_.to_lowercase().replace(" ", "");
  let genstr = match generation {
    None | Some(LATEST_GEN) => get_genstr(LATEST_GEN)?,
    Some(g) => get_genstr(g)?,
  };

  if let Err(_) = open::that(format!(
    "https://www.serebii.net/attackdex{genstr}/{move_}.shtml"
  )) {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!("couldn't open page for {move_}"),
    ));
  }

  Ok(svec!["Opened page successfully."])
}

pub fn open_abilitydex(ability: String) -> Result<Vec<String>, clap::Error> {
  let ability = ability.to_lowercase().replace(" ", "");
  if let Err(_) = open::that(format!(
    "https://www.serebii.net/abilitydex/{ability}.shtml"
  )) {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!("couldn't open page for {ability}"),
    ));
  }

  Ok(svec!["Opened page successfully."])
}

pub fn open_itemdex(item: String) -> Result<Vec<String>, clap::Error> {
  let item = item.to_lowercase().replace(" ", "");
  if let Err(_) = open::that(format!("https://www.serebii.net/itemdex/{item}.shtml")) {
    return Err(cli::error(
      ErrorKind::InvalidValue,
      format!("couldn't open page for {item}"),
    ));
  }

  Ok(svec!["Opened page successfully."])
}

#[cfg(test)]
mod tests {
  use super::*;
}
