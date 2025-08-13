use crate::get_name;
use futures::future;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::*;

pub async fn get_pokemon_name(
  client: &RustemonClient,
  pokemon: &rustemon::model::pokemon::Pokemon,
  lang: &str,
) -> String {
  let forms =
    match future::try_join_all(pokemon.forms.iter().map(async |f| f.follow(&client).await)).await {
      Ok(x) => x,
      Err(_) => return pokemon.name.clone(),
    };

  for form in forms.into_iter() {
    if !form.is_default || form.names.len() == 0 {
      continue;
    }
    for n in form.names.iter() {
      if let Ok(item) = n.language.follow(&client).await
        && item.name == lang
      {
        return n.name.clone();
      }
    }
    break;
  }

  get_name!(follow pokemon.species, client, lang)
}

pub async fn get_pokemon_from_chain(
  client: &RustemonClient,
  pokemon: &str,
  recursive: bool,
) -> Result<Vec<rustemon::model::pokemon::Pokemon>, ()> {
  let mut result = Vec::new();
  let pokemon = match pokemon::get_by_name(pokemon, &client).await {
    Ok(x) => x,
    Err(_) => return Err(()),
  };

  if recursive {
    let species = match pokemon.species.follow(&client).await {
      Ok(x) => x,
      Err(_) => return Err(()),
    };
    if let Some(chain) = species.evolution_chain {
      let chain = match chain.follow(&client).await {
        Ok(x) => x.chain,
        Err(_) => return Err(()),
      };
      if let Ok(x) = pokemon_species::get_by_name(&chain.species.name, &client).await {
        if let Ok(y) = future::try_join_all(
          x.varieties
            .iter()
            .map(async |v| v.pokemon.follow(&client).await),
        )
        .await
        {
          y.into_iter().for_each(|mon| result.push(mon));
        }
      }
      for evo1 in chain.evolves_to.iter() {
        if let Ok(x) = pokemon_species::get_by_name(&evo1.species.name, &client).await {
          if let Ok(y) = future::try_join_all(
            x.varieties
              .iter()
              .map(async |v| v.pokemon.follow(&client).await),
          )
          .await
          {
            y.into_iter().for_each(|mon| result.push(mon));
          }
        }
        for evo2 in evo1.evolves_to.iter() {
          if let Ok(x) = pokemon_species::get_by_name(&evo2.species.name, &client).await {
            if let Ok(y) = future::try_join_all(
              x.varieties
                .iter()
                .map(async |v| v.pokemon.follow(&client).await),
            )
            .await
            {
              y.into_iter().for_each(|mon| result.push(mon));
            }
          }
        }
      }
    }
  } else {
    result.push(pokemon);
  }

  Ok(result)
}

pub fn follow_encounters(
  pokemon: &rustemon::model::pokemon::Pokemon,
) -> Result<Vec<rustemon::model::pokemon::LocationAreaEncounter>, ()> {
  if let Ok(mut url) = ureq::get(pokemon.location_area_encounters.clone()).call()
    && let Ok(body) = url.body_mut().read_to_string()
  {
    let result: Vec<rustemon::model::pokemon::LocationAreaEncounter> =
      match serde_json::from_str(&body) {
        Ok(x) => x,
        Err(_) => {
          return Err(());
        },
      };
    return Ok(result);
  }
  Err(())
}

pub async fn get_evolution_name(
  client: &RustemonClient,
  species: &rustemon::model::resource::NamedApiResource<rustemon::model::pokemon::PokemonSpecies>,
  lang: &str,
  fast: bool,
) -> String {
  if !fast {
    get_name!(follow species, client, lang)
  } else {
    species.name.clone()
  }
}

pub async fn get_evolution_details(
  client: &RustemonClient,
  details: &rustemon::model::evolution::EvolutionDetail,
  lang: &str,
  fast: bool,
) -> Option<String> {
  let mut result = Vec::new();

  // Check item
  if let Some(resource) = &details.item {
    result.push(format!(
      "item: {}",
      if !fast {
        get_name!(follow resource, client, lang)
      } else {
        resource.name.clone()
      },
    ));
  }

  // Check gender
  if let Some(gender) = &details.gender {
    result.push(format!("gender: {gender}"))
  }

  // Check known move
  if let Some(resource) = &details.known_move {
    result.push(format!(
      "known_move: {}",
      if !fast {
        get_name!(follow resource, client, lang)
      } else {
        resource.name.clone()
      },
    ));
  }

  // Check known move type
  if let Some(resource) = &details.known_move_type {
    result.push(format!(
      "known_move_type: {}",
      if !fast {
        get_name!(follow resource, client, lang)
      } else {
        resource.name.clone()
      },
    ));
  }

  // Check location
  if let Some(resource) = &details.location {
    result.push(format!(
      "location: {}",
      if !fast {
        get_name!(follow resource, client, lang)
      } else {
        resource.name.clone()
      },
    ));
  }

  // Check minimum level
  if let Some(val) = &details.min_level {
    result.push(format!("min_level: {val}"));
  }

  // Check minimum happiness
  if let Some(val) = &details.min_happiness {
    result.push(format!("min_happiness: {val}"));
  }

  // Check minimum beauty
  if let Some(val) = &details.min_beauty {
    result.push(format!("min_beauty: {val}"));
  }

  // Check minimum affection
  if let Some(val) = &details.min_affection {
    result.push(format!("min_affection: {val}"));
  }

  // Check overworld rain
  if details.needs_overworld_rain {
    result.push(String::from("needs_overworld_rain"));
  }

  // Check party species
  if let Some(resource) = &details.party_species {
    result.push(format!(
      "party_species: {}",
      if !fast {
        get_name!(follow resource, client, lang)
      } else {
        resource.name.clone()
      },
    ));
  }

  // Check party type
  if let Some(resource) = &details.party_type {
    result.push(format!(
      "party_type: {}",
      if !fast {
        get_name!(follow resource, client, lang)
      } else {
        resource.name.clone()
      },
    ));
  }

  // Check relative physical stats
  if let Some(rel) = &details.relative_physical_stats {
    result.push(format!("relative_physical_stats: {rel}"));
  }

  // Check time of day
  if details.time_of_day.len() != 0 {
    result.push(format!("time_of_day: {}", details.time_of_day));
  }

  // Check trade species
  if let Some(resource) = &details.trade_species {
    result.push(format!(
      "trade_species: {}",
      if !fast {
        get_name!(follow resource, client, lang)
      } else {
        resource.name.clone()
      },
    ));
  }

  // Check upside-down
  if details.turn_upside_down {
    result.push(String::from("turn_upside_down"));
  }

  if result.len() == 0 {
    None
  } else {
    Some(result.join(", "))
  }
}
