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

pub fn open_attackdex() -> Result<Vec<String>, clap::Error> {
  Ok(Vec::new())
}

pub fn open_abilitydex() -> Result<Vec<String>, clap::Error> {
  Ok(Vec::new())
}

pub fn open_itemdex() -> Result<Vec<String>, clap::Error> {
  Ok(Vec::new())
}

#[cfg(test)]
mod tests {
  use super::*;
}
