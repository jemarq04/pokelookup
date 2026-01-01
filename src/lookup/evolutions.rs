use crate::utils::cli::{self, LanguageId};
use crate::utils::helpers;
use crate::{get_name, svec};
use clap::error::ErrorKind;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::*;

pub async fn print_evolutions(
  client: &RustemonClient,
  pokemon: &str,
  fast: bool,
  lang: LanguageId,
  secret: bool,
  all: bool,
) -> Result<Vec<String>, clap::Error> {
  // Create pokemon species resource
  let species = match pokemon_species::get_by_name(&pokemon.replace(" ", "-"), &client).await {
    Ok(x) => x,
    Err(_) => {
      return Err(cli::error(
        ErrorKind::InvalidValue,
        format!("invalid pokemon species: {pokemon}"),
      ));
    },
  };

  // Iterate over evolution chain, if present
  let mut result: Vec<String> = Vec::new();
  let mut force_show_all = false;
  if let Some(chain_resource) = species.evolution_chain {
    // Get evolution chain resource
    let chain = match chain_resource.follow(&client).await {
      Ok(x) => x,
      Err(_) => {
        return Err(cli::error(
          ErrorKind::InvalidValue,
          format!(
            "API error: could not retrieve evolution chain for {}",
            species.name,
          ),
        ));
      },
    };

    if chain.chain.evolves_to.len() == 0 {
      // Record first species name
      result.push(
        helpers::get_evolution_name(
          &client,
          &chain.chain.species,
          &lang.to_string(),
          fast || secret,
        )
        .await,
      );
    }

    // Record exceptional cases
    if svec![
      "rattata", "sandshrew", "vulpix", "meowth", "cubone", "slowpoke", "darumaka"
    ]
    .contains(&chain.chain.species.name)
    {
      force_show_all = true;
    }

    for evo1 in chain.chain.evolves_to.iter() {
      if evo1.evolution_details.len() == 0 {
        result.push(format!(
          "{} -> ??? -> {}",
          helpers::get_evolution_name(
            &client,
            &chain.chain.species,
            &lang.to_string(),
            fast || secret,
          )
          .await,
          helpers::get_evolution_name(&client, &evo1.species, &lang.to_string(), fast || secret)
            .await,
        ));
      } else {
        for method1 in evo1.evolution_details.iter() {
          result.push(format!(
            "{} -> {}",
            helpers::get_evolution_name(
              &client,
              &chain.chain.species,
              &lang.to_string(),
              fast || secret,
            )
            .await,
            if !fast {
              get_name!(follow method1.trigger, client, lang.to_string())
            } else {
              method1.trigger.name.clone()
            },
          ));

          if let Some(details) =
            helpers::get_evolution_details(&client, &method1, &lang.to_string(), fast).await
          {
            result
              .last_mut()
              .unwrap()
              .push_str(&format!(" ({details})"));
          }

          result.last_mut().unwrap().push_str(&format!(
            " -> {}",
            helpers::get_evolution_name(&client, &evo1.species, &lang.to_string(), fast || secret)
              .await,
          ));

          // Check for exceptional cases
          if svec!["sirfetchd", "overqwil", "cursola", "basculegion"].contains(&evo1.species.name) {
            result.insert(
              0,
              if !fast {
                get_name!(follow chain.chain.species, client, lang.to_string())
              } else {
                chain.chain.species.name.clone()
              },
            );
          } else if evo1.species.name == "mr-mime" {
            result.insert(0, result.last().unwrap().clone());
            *result.last_mut().unwrap() = if !fast {
              get_name!(follow evo1.species, client, lang.to_string())
            } else {
              evo1.species.name.clone()
            };
          } else if evo1.species.name == "linoone" {
            result.insert(0, result.last().unwrap().clone());
          }

          // Check for second evolution
          let mut first_evo2 = true;
          let curr_steps = result.last().unwrap().clone();
          for evo2 in evo1.evolves_to.iter() {
            for method2 in evo2.evolution_details.iter() {
              let mut temp_steps: String = format!(
                " -> {}",
                if !fast {
                  get_name!(follow method2.trigger, client, lang.to_string())
                } else {
                  method2.trigger.name.clone()
                },
              );

              if let Some(details) =
                helpers::get_evolution_details(&client, &method2, &lang.to_string(), fast).await
              {
                temp_steps.push_str(&format!(" ({details})"));
              }

              temp_steps.push_str(&format!(
                " -> {}",
                helpers::get_evolution_name(
                  &client,
                  &evo2.species,
                  &lang.to_string(),
                  fast || secret
                )
                .await,
              ));

              if first_evo2 {
                result.last_mut().unwrap().push_str(&temp_steps);
                first_evo2 = false;
              } else {
                result.push(format!("{}{}", curr_steps, temp_steps));
              }
            }
          }
        }
      }
    }
  } else {
    // No chain found => record species name to final result
    result.push(if !fast {
      get_name!(species, client, lang.to_string())
    } else {
      species.name.clone()
    });
  }

  // Only provide newest evolution methods
  if !all && !force_show_all {
    let mut temp = Vec::new();
    let mut prev_names = Vec::new();
    let mut prev_line = String::new();

    for line in result.iter() {
      // Get list of pokemon names
      let mut names: Vec<String> = line.split(" -> ").map(|s| s.to_string()).collect();
      for i in (1..4).step_by(2).rev() {
        if names.len() > i {
          names.remove(i);
        }
      }

      // Add most recent evolution method to temp vector
      if names == prev_names {
        prev_line = line.clone();
      } else {
        if !prev_line.is_empty() {
          temp.push(prev_line);
        }
        prev_names = names.clone();
        prev_line = line.clone();
      }
    }
    temp.push(prev_line);

    // Set output to temp vector
    result = temp;
  }

  // Hide pokemon names, if desired
  if secret {
    let mut temp = Vec::new();

    for line in result.iter() {
      let mut info: Vec<String> = line.split(" -> ").map(|s| s.to_string()).collect();
      for i in (0..info.len()).step_by(2) {
        info[i] = String::from("MON");
      }
      temp.push(info.join(" -> "));
    }

    result = temp;
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_evolutions() {
    let client = RustemonClient::default();

    let success = vec![
      vec![
        "eevee -> use-item (item: water-stone) -> vaporeon",
        "eevee -> use-item (item: thunder-stone) -> jolteon",
        "eevee -> use-item (item: fire-stone) -> flareon",
        "eevee -> level-up (min_happiness: 160, time_of_day: day) -> espeon",
        "eevee -> level-up (min_happiness: 160, time_of_day: night) -> umbreon",
        "eevee -> level-up (location: eterna-forest) -> leafeon",
        "eevee -> level-up (location: pinwheel-forest) -> leafeon",
        "eevee -> level-up (location: kalos-route-20) -> leafeon",
        "eevee -> use-item (item: leaf-stone) -> leafeon",
        "eevee -> level-up (location: sinnoh-route-217) -> glaceon",
        "eevee -> level-up (location: twist-mountain) -> glaceon",
        "eevee -> level-up (location: frost-cavern) -> glaceon",
        "eevee -> use-item (item: ice-stone) -> glaceon",
        "eevee -> level-up (known_move_type: fairy, min_affection: 2) -> sylveon",
        "eevee -> level-up (known_move_type: fairy, min_happiness: 160) -> sylveon",
      ],
      vec![
        "Eevee -> Use item (item: Water Stone) -> Vaporeon",
        "Eevee -> Use item (item: Thunder Stone) -> Jolteon",
        "Eevee -> Use item (item: Fire Stone) -> Flareon",
        "Eevee -> Level up (min_happiness: 160, time_of_day: day) -> Espeon",
        "Eevee -> Level up (min_happiness: 160, time_of_day: night) -> Umbreon",
        "Eevee -> Level up (location: Eterna Forest) -> Leafeon",
        "Eevee -> Level up (location: Pinwheel Forest) -> Leafeon",
        "Eevee -> Level up (location: Route 20) -> Leafeon",
        "Eevee -> Use item (item: Leaf Stone) -> Leafeon",
        "Eevee -> Level up (location: Route 217) -> Glaceon",
        "Eevee -> Level up (location: Twist Mountain) -> Glaceon",
        "Eevee -> Level up (location: Frost Cavern) -> Glaceon",
        "Eevee -> Use item (item: Ice Stone) -> Glaceon",
        "Eevee -> Level up (known_move_type: Fairy, min_affection: 2) -> Sylveon",
        "Eevee -> Level up (known_move_type: Fairy, min_happiness: 160) -> Sylveon",
      ],
    ];

    for (idx, vals) in success.into_iter().enumerate() {
      let pokemon = String::from("Eevee");
      let fast = idx == 0;
      let lang = LanguageId::En;
      let secret = false;
      let all = true;

      match print_evolutions(&client, &pokemon, fast, lang, secret, all).await {
        Ok(res) => assert_eq!(res, vals),
        Err(err) => panic!("{}", err.render()),
      }
    }
  }

  #[tokio::test]
  async fn test_evolutions_secret() {
    let client = RustemonClient::default();

    let success = vec![
      "MON -> use-item (item: water-stone) -> MON",
      "MON -> use-item (item: thunder-stone) -> MON",
      "MON -> use-item (item: fire-stone) -> MON",
      "MON -> level-up (min_happiness: 160, time_of_day: day) -> MON",
      "MON -> level-up (min_happiness: 160, time_of_day: night) -> MON",
      "MON -> level-up (location: eterna-forest) -> MON",
      "MON -> level-up (location: pinwheel-forest) -> MON",
      "MON -> level-up (location: kalos-route-20) -> MON",
      "MON -> use-item (item: leaf-stone) -> MON",
      "MON -> level-up (location: sinnoh-route-217) -> MON",
      "MON -> level-up (location: twist-mountain) -> MON",
      "MON -> level-up (location: frost-cavern) -> MON",
      "MON -> use-item (item: ice-stone) -> MON",
      "MON -> level-up (known_move_type: fairy, min_affection: 2) -> MON",
      "MON -> level-up (known_move_type: fairy, min_happiness: 160) -> MON",
    ];

    let pokemon = String::from("Eevee");
    let fast = true;
    let lang = LanguageId::En;
    let secret = true;
    let all = true;

    match print_evolutions(&client, &pokemon, fast, lang, secret, all).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }

  #[tokio::test]
  async fn test_evolutions_language() {
    let client = RustemonClient::default();

    let success = vec![
      "Eevee -> use-item (item: Piedra Agua) -> Vaporeon",
      "Eevee -> use-item (item: Piedra Trueno) -> Jolteon",
      "Eevee -> use-item (item: Piedra Fuego) -> Flareon",
      "Eevee -> level-up (min_happiness: 160, time_of_day: day) -> Espeon",
      "Eevee -> level-up (min_happiness: 160, time_of_day: night) -> Umbreon",
      "Eevee -> level-up (location: eterna-forest) -> Leafeon",
      "Eevee -> level-up (location: pinwheel-forest) -> Leafeon",
      "Eevee -> level-up (location: Ruta 20) -> Leafeon",
      "Eevee -> use-item (item: Piedra Hoja) -> Leafeon",
      "Eevee -> level-up (location: sinnoh-route-217) -> Glaceon",
      "Eevee -> level-up (location: twist-mountain) -> Glaceon",
      "Eevee -> level-up (location: Gruta Helada) -> Glaceon",
      "Eevee -> use-item (item: Piedra Hielo) -> Glaceon",
      "Eevee -> level-up (known_move_type: Hada, min_affection: 2) -> Sylveon",
      "Eevee -> level-up (known_move_type: Hada, min_happiness: 160) -> Sylveon",
    ];

    let pokemon = String::from("Eevee");
    let fast = false;
    let lang = LanguageId::Es;
    let secret = false;
    let all = true;

    match print_evolutions(&client, &pokemon, fast, lang, secret, all).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }

  #[tokio::test]
  async fn test_evolutions_no_all() {
    let client = RustemonClient::default();

    let success = vec![
      "Eevee -> Use item (item: Water Stone) -> Vaporeon",
      "Eevee -> Use item (item: Thunder Stone) -> Jolteon",
      "Eevee -> Use item (item: Fire Stone) -> Flareon",
      "Eevee -> Level up (min_happiness: 160, time_of_day: day) -> Espeon",
      "Eevee -> Level up (min_happiness: 160, time_of_day: night) -> Umbreon",
      "Eevee -> Use item (item: Leaf Stone) -> Leafeon",
      "Eevee -> Use item (item: Ice Stone) -> Glaceon",
      "Eevee -> Level up (known_move_type: Fairy, min_happiness: 160) -> Sylveon",
    ];

    let pokemon = String::from("Eevee");
    let fast = false;
    let lang = LanguageId::En;
    let secret = false;
    let all = false;

    match print_evolutions(&client, &pokemon, fast, lang, secret, all).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }

  #[tokio::test]
  async fn test_evolutions_exceptions() {
    let client = RustemonClient::default();

    let success = vec![
      "Farfetch’d",
      "Farfetch’d -> Land three critical hits in a battle -> Sirfetch’d",
    ];

    let pokemon = String::from("Farfetchd");
    let fast = false;
    let lang = LanguageId::En;
    let secret = false;
    let all = false;

    match print_evolutions(&client, &pokemon, fast, lang, secret, all).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }

  #[tokio::test]
  async fn test_evolutions_regional_forms() {
    let client = RustemonClient::default();

    let success = vec![
      "Rattata -> Level up (min_level: 20) -> Raticate",
      "Rattata -> Level up (min_level: 20, time_of_day: night) -> Raticate",
    ];

    let pokemon = String::from("Rattata");
    let fast = false;
    let lang = LanguageId::En;
    let secret = false;
    let all = false;

    match print_evolutions(&client, &pokemon, fast, lang, secret, all).await {
      Ok(res) => assert_eq!(res, success),
      Err(err) => panic!("{}", err.render()),
    }
  }
}
