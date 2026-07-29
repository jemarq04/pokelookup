use crate::get_name;
use futures::future;
use rustemon::Follow;
use rustemon::client::RustemonClient;
use rustemon::pokemon::{pokemon, pokemon_species};

pub async fn get_pokemon_name(
  client: &RustemonClient,
  pokemon: &rustemon::model::pokemon::Pokemon,
  lang: &str,
) -> String {
  let Ok(forms) =
    future::try_join_all(pokemon.forms.iter().map(async |f| f.follow(client).await)).await
  else {
    return pokemon.name.clone();
  };

  for form in forms {
    if !form.is_default || form.names.is_empty() {
      continue;
    }
    for n in &form.names {
      if let Ok(item) = n.language.follow(client).await
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
  let Ok(pokemon) = pokemon::get_by_name(pokemon, client).await else {
    return Err(());
  };

  if recursive {
    let Ok(species) = pokemon.species.follow(client).await else {
      return Err(());
    };
    if let Some(chain) = species.evolution_chain {
      let Ok(chain) = chain.follow(client).await else {
        return Err(());
      };
      if let Ok(x) = pokemon_species::get_by_name(&chain.chain.species.name, client).await
        && let Ok(y) = future::try_join_all(
          x.varieties
            .iter()
            .map(async |v| v.pokemon.follow(client).await),
        )
        .await
      {
        for mon in y {
          result.push(mon);
        }
      }
      for evo1 in &chain.chain.evolves_to {
        if let Ok(x) = pokemon_species::get_by_name(&evo1.species.name, client).await
          && let Ok(y) = future::try_join_all(
            x.varieties
              .iter()
              .map(async |v| v.pokemon.follow(client).await),
          )
          .await
        {
          for mon in y {
            result.push(mon);
          }
        }
        for evo2 in &evo1.evolves_to {
          if let Ok(x) = pokemon_species::get_by_name(&evo2.species.name, client).await
            && let Ok(y) = future::try_join_all(
              x.varieties
                .iter()
                .map(async |v| v.pokemon.follow(client).await),
            )
            .await
          {
            for mon in y {
              result.push(mon);
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
  secret: bool,
) -> String {
  if secret {
    String::from("MON")
  } else if fast {
    species.name.clone()
  } else {
    get_name!(follow species, client, lang)
  }
}

pub async fn get_evolution_details(
  client: &RustemonClient,
  details: &rustemon::model::evolution::EvolutionDetail,
  lang: &str,
  fast: bool,
) -> Option<String> {
  let mut result = Vec::new();

  // Check version group
  if !details.is_default {
    let version_group = details.version_group.follow(client).await.unwrap();
    let mut versions = Vec::new();
    for version_resource in &version_group.versions {
      versions.push(if fast {
        version_resource.name.clone()
      } else {
        get_name!(follow version_resource, client, lang)
      });
    }
    result.push(format!("versions: {}", versions.join("/")));
  }

  // Check item
  if let Some(resource) = &details.item {
    result.push(format!(
      "item: {}",
      if fast {
        resource.name.clone()
      } else {
        get_name!(follow resource, client, lang)
      },
    ));
  }

  // Check held item
  if let Some(resource) = &details.held_item {
    result.push(format!(
      "held_item: {}",
      if fast {
        resource.name.clone()
      } else {
        get_name!(follow resource, client, lang)
      },
    ));
  }

  // Check gender
  if let Some(gender) = &details.gender {
    result.push(format!("gender: {gender}"));
  }

  // Check known move
  if let Some(resource) = &details.known_move {
    result.push(format!(
      "known_move: {}",
      if fast {
        resource.name.clone()
      } else {
        get_name!(follow resource, client, lang)
      },
    ));
  }

  // Check known move type
  if let Some(resource) = &details.known_move_type {
    result.push(format!(
      "known_move_type: {}",
      if fast {
        resource.name.clone()
      } else {
        get_name!(follow resource, client, lang)
      },
    ));
  }

  // Check used move
  if let Some(resource) = &details.used_move {
    result.push(format!(
      "used_move: {}",
      if fast {
        resource.name.clone()
      } else {
        get_name!(follow resource, client, lang)
      },
    ));
  }

  // Check location
  if let Some(resource) = &details.location {
    result.push(format!(
      "location: {}",
      if fast {
        resource.name.clone()
      } else {
        get_name!(follow resource, client, lang)
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

  // Check special rock requirement
  if details.near_special_rock {
    result.push(String::from("near_special_rock"));
  }

  // Check multiplayer requirement
  if details.needs_multiplayer {
    result.push(String::from("needs_multiplayer"));
  }

  // Check overworld rain
  if details.needs_overworld_rain {
    result.push(String::from("needs_overworld_rain"));
  }

  // Check party species
  if let Some(resource) = &details.party_species {
    result.push(format!(
      "party_species: {}",
      if fast {
        resource.name.clone()
      } else {
        get_name!(follow resource, client, lang)
      },
    ));
  }

  // Check party type
  if let Some(resource) = &details.party_type {
    result.push(format!(
      "party_type: {}",
      if fast {
        resource.name.clone()
      } else {
        get_name!(follow resource, client, lang)
      },
    ));
  }

  // Check relative physical stats
  if let Some(rel) = &details.relative_physical_stats {
    result.push(format!("relative_physical_stats: {rel}"));
  }

  // Check time of day
  if !details.time_of_day.is_empty() {
    result.push(format!("time_of_day: {}", details.time_of_day));
  }

  // Check trade species
  if let Some(resource) = &details.trade_species {
    result.push(format!(
      "trade_species: {}",
      if fast {
        resource.name.clone()
      } else {
        get_name!(follow resource, client, lang)
      },
    ));
  }

  // Check upside-down
  if details.turn_upside_down {
    result.push(String::from("turn_upside_down"));
  }

  // Check region
  if let Some(resource) = &details.region {
    result.push(format!(
      "region: {}",
      if fast {
        resource.name.clone()
      } else {
        get_name!(follow resource, client, lang)
      },
    ));
  }

  // Check minimum move count
  if let Some(val) = &details.min_move_count {
    result.push(format!("min_move_count: {val}"));
  }

  // Check minimum steps taken
  if let Some(val) = &details.min_steps {
    result.push(format!("min_steps: {val}"));
  }

  // Check minimum damage taken
  if let Some(val) = &details.min_damage_taken {
    result.push(format!("min_damage_taken: {val}"));
  }

  if result.is_empty() {
    None
  } else {
    Some(result.join(", "))
  }
}
