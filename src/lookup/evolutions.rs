use crate::get_name;
use crate::utils::cli;
use crate::utils::enums::LanguageId;
use crate::utils::helpers;
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
  let species = match pokemon_species::get_by_name(&pokemon.replace(" ", "-"), client).await {
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
  if let Some(chain_resource) = species.evolution_chain {
    // Get evolution chain resource
    let chain = match chain_resource.follow(client).await {
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

    if chain.chain.evolves_to.is_empty() {
      // Record species name
      result.push(
        helpers::get_evolution_name(
          client,
          &chain.chain.species,
          &lang.to_string(),
          fast,
          secret,
        )
        .await,
      );
    }

    for evo1 in chain.chain.evolves_to.iter() {
      if evo1.evolution_details.is_empty() {
        // Unknown evolution detail
        result.push(format!(
          "{} -> ??? -> {}",
          helpers::get_evolution_name(
            client,
            &chain.chain.species,
            &lang.to_string(),
            fast,
            secret,
          )
          .await,
          helpers::get_evolution_name(client, &evo1.species, &lang.to_string(), fast, secret).await,
        ));
      } else {
        for details1 in evo1.evolution_details.iter() {
          if !all && !details1.is_default {
            continue;
          }

          result.push(format!(
            "{} -> {}",
            if let Some(base_form_resource) = &details1.base_form {
              let base_form = base_form_resource.follow(client).await.unwrap();
              helpers::get_pokemon_name(client, &base_form, &lang.to_string()).await
            } else {
              helpers::get_evolution_name(
                client,
                &chain.chain.species,
                &lang.to_string(),
                fast,
                secret,
              )
              .await
            },
            if !fast {
              get_name!(follow details1.trigger, client, lang.to_string())
            } else {
              details1.trigger.name.clone()
            },
          ));

          if let Some(details_str) =
            helpers::get_evolution_details(client, details1, &lang.to_string(), fast || secret)
              .await
          {
            result
              .last_mut()
              .unwrap()
              .push_str(&format!(" ({details_str})"));
          }

          result.last_mut().unwrap().push_str(&format!(
            " -> {}",
            if let Some(evolved_form_resource) = &details1.evolved_form {
              let evolved_form = evolved_form_resource.follow(client).await.unwrap();
              helpers::get_pokemon_name(client, &evolved_form, &lang.to_string()).await
            } else {
              helpers::get_evolution_name(client, &evo1.species, &lang.to_string(), fast, secret)
                .await
            }
          ));

          // Check for second evolution
          let mut first_evo2 = true;
          let curr_steps = result.last().unwrap().clone();
          for evo2 in evo1.evolves_to.iter() {
            for details2 in evo2.evolution_details.iter() {
              if !all && !details2.is_default {
                continue;
              }
              let mut temp_steps: String = format!(
                " -> {}",
                if !fast {
                  get_name!(follow details2.trigger, client, lang.to_string())
                } else {
                  details2.trigger.name.clone()
                }
              );

              if let Some(details_str) =
                helpers::get_evolution_details(client, details2, &lang.to_string(), fast).await
              {
                temp_steps.push_str(&format!(" ({details_str})"));
              }

              temp_steps.push_str(&format!(
                " -> {}",
                if let Some(evolved_form_resource) = &details2.evolved_form {
                  let evolved_form = evolved_form_resource.follow(client).await.unwrap();
                  helpers::get_pokemon_name(client, &evolved_form, &lang.to_string()).await
                } else {
                  helpers::get_evolution_name(
                    client,
                    &evo2.species,
                    &lang.to_string(),
                    fast,
                    secret,
                  )
                  .await
                }
              ));

              if first_evo2 {
                result.last_mut().unwrap().push_str(&temp_steps);
                first_evo2 = false;
              } else {
                result.push(format!("{curr_steps}{temp_steps}"));
              }
            }
          }
        }
      }
    }
  } else {
    // No chain found => record species name to final result
    result.push(if secret {
      String::from("???")
    } else if fast {
      species.name.clone()
    } else {
      get_name!(species, client, lang.to_string())
    });
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
        "eevee -> level-up (versions: diamond/pearl, location: eterna-forest, near_special_rock) -> leafeon",
        "eevee -> level-up (versions: black/white, location: pinwheel-forest, near_special_rock) -> leafeon",
        "eevee -> level-up (versions: x/y, location: kalos-route-20, near_special_rock) -> leafeon",
        "eevee -> use-item (item: leaf-stone) -> leafeon",
        "eevee -> level-up (versions: omega-ruby/alpha-sapphire, location: petalburg-woods, near_special_rock) -> leafeon",
        "eevee -> level-up (versions: sun/moon, location: lush-jungle, near_special_rock) -> leafeon",
        "eevee -> level-up (versions: diamond/pearl, location: sinnoh-route-217, near_special_rock) -> glaceon",
        "eevee -> level-up (versions: black/white, location: twist-mountain, near_special_rock) -> glaceon",
        "eevee -> level-up (versions: x/y, location: frost-cavern, near_special_rock) -> glaceon",
        "eevee -> use-item (item: ice-stone) -> glaceon",
        "eevee -> level-up (versions: omega-ruby/alpha-sapphire, location: shoal-cave, near_special_rock) -> glaceon",
        "eevee -> level-up (versions: sun/moon, location: mount-lanakila, near_special_rock) -> glaceon",
        "eevee -> level-up (versions: x/y, known_move_type: fairy, min_affection: 2) -> sylveon",
        "eevee -> level-up (known_move_type: fairy, min_happiness: 160) -> sylveon",
      ],
      vec![
        "Eevee -> Use item (item: Water Stone) -> Vaporeon",
        "Eevee -> Use item (item: Thunder Stone) -> Jolteon",
        "Eevee -> Use item (item: Fire Stone) -> Flareon",
        "Eevee -> Level up (min_happiness: 160, time_of_day: day) -> Espeon",
        "Eevee -> Level up (min_happiness: 160, time_of_day: night) -> Umbreon",
        "Eevee -> Level up (versions: Diamond/Pearl, location: Eterna Forest, near_special_rock) -> Leafeon",
        "Eevee -> Level up (versions: Black/White, location: Pinwheel Forest, near_special_rock) -> Leafeon",
        "Eevee -> Level up (versions: X/Y, location: Route 20, near_special_rock) -> Leafeon",
        "Eevee -> Use item (item: Leaf Stone) -> Leafeon",
        "Eevee -> Level up (versions: Omega Ruby/Alpha Sapphire, location: Petalburg Woods, near_special_rock) -> Leafeon",
        "Eevee -> Level up (versions: Sun/Moon, location: Lush Jungle, near_special_rock) -> Leafeon",
        "Eevee -> Level up (versions: Diamond/Pearl, location: Route 217, near_special_rock) -> Glaceon",
        "Eevee -> Level up (versions: Black/White, location: Twist Mountain, near_special_rock) -> Glaceon",
        "Eevee -> Level up (versions: X/Y, location: Frost Cavern, near_special_rock) -> Glaceon",
        "Eevee -> Use item (item: Ice Stone) -> Glaceon",
        "Eevee -> Level up (versions: Omega Ruby/Alpha Sapphire, location: Shoal Cave, near_special_rock) -> Glaceon",
        "Eevee -> Level up (versions: Sun/Moon, location: Mount Lanakila, near_special_rock) -> Glaceon",
        "Eevee -> Level up (versions: X/Y, known_move_type: Fairy, min_affection: 2) -> Sylveon",
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
      "MON -> level-up (versions: diamond/pearl, location: eterna-forest, near_special_rock) -> MON",
      "MON -> level-up (versions: black/white, location: pinwheel-forest, near_special_rock) -> MON",
      "MON -> level-up (versions: x/y, location: kalos-route-20, near_special_rock) -> MON",
      "MON -> use-item (item: leaf-stone) -> MON",
      "MON -> level-up (versions: omega-ruby/alpha-sapphire, location: petalburg-woods, near_special_rock) -> MON",
      "MON -> level-up (versions: sun/moon, location: lush-jungle, near_special_rock) -> MON",
      "MON -> level-up (versions: diamond/pearl, location: sinnoh-route-217, near_special_rock) -> MON",
      "MON -> level-up (versions: black/white, location: twist-mountain, near_special_rock) -> MON",
      "MON -> level-up (versions: x/y, location: frost-cavern, near_special_rock) -> MON",
      "MON -> use-item (item: ice-stone) -> MON",
      "MON -> level-up (versions: omega-ruby/alpha-sapphire, location: shoal-cave, near_special_rock) -> MON",
      "MON -> level-up (versions: sun/moon, location: mount-lanakila, near_special_rock) -> MON",
      "MON -> level-up (versions: x/y, known_move_type: fairy, min_affection: 2) -> MON",
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
      "Eevee -> level-up (versions: Diamante/Perla, location: eterna-forest, near_special_rock) -> Leafeon",
      "Eevee -> level-up (versions: Negro/Blanco, location: pinwheel-forest, near_special_rock) -> Leafeon",
      "Eevee -> level-up (versions: X/Y, location: Ruta 20, near_special_rock) -> Leafeon",
      "Eevee -> use-item (item: Piedra Hoja) -> Leafeon",
      "Eevee -> level-up (versions: Rubí Omega/Zafiro Alfa, location: Bosque Petalia, near_special_rock) -> Leafeon",
      "Eevee -> level-up (versions: Sol/Luna, location: Jungla Umbría, near_special_rock) -> Leafeon",
      "Eevee -> level-up (versions: Diamante/Perla, location: sinnoh-route-217, near_special_rock) -> Glaceon",
      "Eevee -> level-up (versions: Negro/Blanco, location: twist-mountain, near_special_rock) -> Glaceon",
      "Eevee -> level-up (versions: X/Y, location: Gruta Helada, near_special_rock) -> Glaceon",
      "Eevee -> use-item (item: Piedra Hielo) -> Glaceon",
      "Eevee -> level-up (versions: Rubí Omega/Zafiro Alfa, location: Cueva Cardumen, near_special_rock) -> Glaceon",
      "Eevee -> level-up (versions: Sol/Luna, location: Monte Lanakila, near_special_rock) -> Glaceon",
      "Eevee -> level-up (versions: X/Y, known_move_type: Hada, min_affection: 2) -> Sylveon",
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
  async fn test_evolutions_regional_forms() {
    let client = RustemonClient::default();

    let success = vec![
      "Rattata -> Level up (min_level: 20) -> Raticate",
      "Alolan Rattata -> Level up (min_level: 20, time_of_day: night) -> Alolan Raticate",
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
