use super::pokedex::Pokedex;
use crate::utils::cli;
use clap::ValueEnum;
use clap::error::ErrorKind;

const LATEST_GEN: i64 = 9;

pub fn open_pokedex(pokemon: String, generation: Option<i64>) -> Result<String, clap::Error> {
  let pokemon = pokemon.to_lowercase().replace(" ", "");
  let url = match generation {
    None | Some(0) => format!("https://www.serebii.net/pokemon/{pokemon}/"),
    Some(g) => match g {
      g @ 8..=LATEST_GEN => {
        format!(
          "https://www.serebii.net/pokedex-{}/{pokemon}/",
          match g {
            8 => "swsh",
            9 => "sv",
            _ => unreachable!(),
          }
        )
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
        format!(
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
        )
      },
      _ => {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!("invalid generation: {g}"),
        ));
      },
    },
  };

  Ok(url)
}

pub fn open_pokearth(
  region: String,
  area: Option<String>,
  generation: Option<i64>,
) -> Result<String, clap::Error> {
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

  Ok(url)
}

pub fn open_attackdex(move_: String, generation: Option<i64>) -> Result<String, clap::Error> {
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

  Ok(format!(
    "https://www.serebii.net/attackdex{genstr}/{move_}.shtml"
  ))
}

pub fn open_abilitydex(ability: String) -> Result<String, clap::Error> {
  let ability = ability.to_lowercase().replace(" ", "");
  Ok(format!(
    "https://www.serebii.net/abilitydex/{ability}.shtml"
  ))
}

pub fn open_itemdex(item: String) -> Result<String, clap::Error> {
  let item = item.to_lowercase().replace(" ", "");
  Ok(format!("https://www.serebii.net/itemdex/{item}.shtml"))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::svec;
  use rustemon::{Follow, client::RustemonClient};

  #[tokio::test]
  async fn test_latest_generation() {
    let client = RustemonClient::default();

    let mut all_generations = rustemon::games::generation::get_all_entries(&client)
      .await
      .unwrap();
    match all_generations.pop() {
      Some(generation_resource) => {
        let generation = generation_resource.follow(&client).await.unwrap();
        assert_eq!(generation.id, LATEST_GEN)
      },
      None => panic!("Could not retrieve generation resources"),
    };
  }

  // Test each function with and without generation, trying to use names with upper-case and spaces
  #[test]
  fn test_pokedex() {
    match open_pokedex(String::from("Iron Treads"), None) {
      Ok(url) => assert_eq!(url, "https://www.serebii.net/pokemon/irontreads/"),
      Err(err) => panic!("{}", err.render()),
    }

    let success = svec![
      "https://www.serebii.net/pokedex-rs/025.shtml",
      "https://www.serebii.net/pokedex-sv/pikachu/",
    ];

    for (idx, val) in success.iter().enumerate() {
      match open_pokedex(String::from("Pikachu"), Some(6 * idx as i64 + 3)) {
        Ok(url) => assert_eq!(url, *val),
        Err(err) => panic!("{}", err.render()),
      }
    }
  }

  #[test]
  fn test_pokearth() {
    match open_pokearth(String::from("Sinnoh"), None, None) {
      Ok(url) => assert_eq!(url, "https://www.serebii.net/pokearth/sinnoh/"),
      Err(err) => panic!("{}", err.render()),
    }

    match open_pokearth(
      String::from("Johto"),
      Some(String::from("Olivine City")),
      Some(2),
    ) {
      Ok(url) => assert_eq!(
        url,
        "https://www.serebii.net/pokearth/johto/2nd/olivinecity.shtml"
      ),
      Err(err) => panic!("{}", err.render()),
    }
  }

  #[test]
  fn test_attackdex() {
    match open_attackdex(String::from("Thunder Wave"), None) {
      Ok(url) => assert_eq!(
        url,
        "https://www.serebii.net/attackdex-sv/thunderwave.shtml"
      ),
      Err(err) => panic!("{}", err.render()),
    }

    match open_attackdex(String::from("Thunder Wave"), Some(5)) {
      Ok(url) => assert_eq!(
        url,
        "https://www.serebii.net/attackdex-bw/thunderwave.shtml"
      ),
      Err(err) => panic!("{}", err.render()),
    }
  }

  #[test]
  fn test_abilitydex() {
    match open_abilitydex(String::from("Tablets of Ruin")) {
      Ok(url) => assert_eq!(
        url,
        "https://www.serebii.net/abilitydex/tabletsofruin.shtml"
      ),
      Err(err) => panic!("{}", err.render()),
    }
  }

  #[test]
  fn test_itemdex() {
    match open_itemdex(String::from("Thunder Stone")) {
      Ok(url) => assert_eq!(url, "https://www.serebii.net/itemdex/thunderstone.shtml"),
      Err(err) => panic!("{}", err.render()),
    }
  }
}
