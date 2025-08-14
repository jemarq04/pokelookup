use super::pokedex::Pokedex;
use crate::svec;
use crate::utils::cli;
use clap::ValueEnum;
use clap::error::ErrorKind;

pub fn open_pokedex(pokemon: String, generation: i64) -> Result<Vec<String>, clap::Error> {
  match generation {
    0 => {
      if let Err(_) = open::that(format!("https://www.serebii.net/pokemon/{pokemon}")) {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!("couldn't open page for {pokemon}"),
        ));
      }
    },
    g @ 8..=cli::LATEST_GEN => {
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
        format!("invalid generation: {generation}"),
      ));
    },
  }
  Ok(svec!["Opened page successfully."])
}

pub fn open_pokearth() -> Result<Vec<String>, clap::Error> {
  Ok(Vec::new())
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
