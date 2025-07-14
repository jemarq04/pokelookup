use clap::{Parser,Subcommand};
use futures::future;
use rustemon::pokemon::*;
use rustemon::Follow;

#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    #[command(subcommand)]
    command: SubArgs,
}

#[derive(Subcommand,Debug)]
enum SubArgs {
    #[command(name="types", about="look up the types of a pokemon")]
    TypeCmd {
        #[arg(help="name of pokemon")]
        pokemon: String,

        #[arg(short, long, help="skip API requests for formatted names")]
        fast: bool,

        //#[arg(short, help="recursively check evolution chain")]
        //recursive: bool,
    },

    #[command(name="abilities", about="look up the abilities of a pokemon")]
    AbilityCmd {
        #[arg(help="name of pokemon")]
        pokemon: String,

        #[arg(short, long, help="skip API requests for formatted names")]
        fast: bool,
    },

    #[command(name="moves", about="look up the level-up movesets of a pokemon")]
    MoveCmd {
        #[arg(help="name of pokemon")]
        pokemon: String,

        #[arg(short, long, help="skip API requests for formatted names")]
        fast: bool,

        #[arg(short, long, default_value_t=String::from("scarlet-violet"), help="version group name")]
        version: String,

        #[arg(short, long, help="request default moveset at given level")]
        level: Option<i64>,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    match args.command {
        SubArgs::TypeCmd{..} => print_types(&args.command).await,
        SubArgs::AbilityCmd{..} => print_abilities(&args.command).await,
        SubArgs::MoveCmd{..} => print_moves(&args.command).await,
        //_ => panic!("error: not yet implemented"),
    };
}

async fn get_name(
    client: &rustemon::client::RustemonClient, 
    names: &Vec<rustemon::model::resource::Name>, 
    lang: &str,
    ) -> Result<String, ()>
{
    for n in names.iter() {
        if let Ok(x) = n.language.follow(&client).await && x.name == lang {
            return Ok(n.name.clone());
        }
    }
    Err(())
}

async fn print_types(args: &SubArgs) {
    let SubArgs::TypeCmd{pokemon, fast, ..} = args else {
        return;
    };

    // Create client
    let client = rustemon::client::RustemonClient::default();

    // Create pokemon resource
    let mon_resource = match pokemon::get_by_name(&pokemon, &client).await {
        Ok(x) => x,
        Err(_) => panic!("error: could not find pokemon {}", pokemon),
    };

    // Get types
    let types = match future::try_join_all(
        mon_resource.types.iter().map(async |t| t.type_.follow(&client).await)
    ).await {
        Ok(x) => x,
        Err(_) => panic!("error: could not retrive types for pokemon {}", pokemon),
    };

    // Print English names
    let result = if *fast {
        types.into_iter().map(|t| t.name).collect()
    }
    else {
        match future::try_join_all(types.into_iter().map(|t| t.names).map(
            async |names| get_name(&client, &names, "en").await
        )).await {
            Ok(x) => x,
            Err(_) => panic!("error: could not find English names for types"),
        }
    };

    println!("{}: {:?}", pokemon, result);
}

async fn print_abilities(args: &SubArgs) {
    let SubArgs::AbilityCmd{pokemon, fast, ..} = args else {
        return;
    };

    // Create client
    let client = rustemon::client::RustemonClient::default();

    // Create pokemon resource
    let mon_resource = match pokemon::get_by_name(&pokemon, &client).await {
        Ok(x) => x,
        Err(_) => panic!("error: could not find pokemon {}", pokemon),
    };

    // Create struct to store ability
    struct Ability {
        hidden: bool,
        ability: rustemon::model::pokemon::Ability,
    }

    // Get abilities
    let abilities = match future::try_join_all(
        mon_resource.abilities.iter().map(async |a| {
            match a.ability.follow(&client).await {
                Ok(x) => Ok(Ability{hidden: a.is_hidden, ability: x}),
                Err(_) => Err(()),
            }
        })
    ).await {
        Ok(x) => x,
        Err(_) => panic!("error: could not retrive abilities for pokemon {}", pokemon),
    };

    // Print English names
    let mut result = Vec::new();
    for ab in abilities.into_iter() {
        if *fast {
            result.push(ab.ability.name.clone() + if ab.hidden {" (Hidden)"} else {""});
        }
        else if let Ok(x) = get_name(&client, &ab.ability.names, "en").await {
            result.push(x + if ab.hidden {" (Hidden)"} else {""});
        }
    }
    println!("{}: {:?}", pokemon, result);
}

async fn print_moves(args: &SubArgs) {
    let SubArgs::MoveCmd{pokemon, fast, version, level, ..} = args else {
        return;
    };
    
    // Create client
    let client = rustemon::client::RustemonClient::default();

    // Create pokemon resource
    let mon_resource = match pokemon::get_by_name(&pokemon, &client).await {
        Ok(x) => x,
        Err(_) => panic!("error: could not find pokemon {}", pokemon),
    };
    
    // Create struct to store move
    #[derive(Debug)]
    struct Move {
        name: String,
        level: i64,
    }

    // Get full learnset
    let mut moves = Vec::new();
    for move_resource in mon_resource.moves.iter() {
        for details in move_resource.version_group_details.iter() {
            if details.move_learn_method.name == "level-up" && details.version_group.name == *version {
                match *level {
                    Some(x) if details.level_learned_at > x => {},
                    _ => {
                        if *fast {
                            moves.push(Move{
                                name: move_resource.move_.name.clone(),
                                level: details.level_learned_at,
                            });
                        }
                        else if let Ok(x) = move_resource.move_.follow(&client).await {
                            if let Ok(y) = get_name(&client, &x.names, "en").await {
                                moves.push(Move{
                                    name: y,
                                    level: details.level_learned_at,
                                });
                            }
                        }
                        else {
                            panic!("error: could not find move {}", move_resource.move_.name);
                        }
                    },
                };
            }
        }
    }
    // Sort moves by descending level
    moves.sort_by(|m, n| {n.level.cmp(&m.level)});

    // Print result
    let mut result = if let Some(_) = *level {
        moves.iter().take(4).collect::<Vec<_>>()
    }
    else {
        moves.iter().collect::<Vec<_>>()
    };
    result.reverse();
    let _temp: Vec<String> = result.iter().map(|x| x.name.clone()).collect();
    println!("{}: {:?}", pokemon, result);
}
