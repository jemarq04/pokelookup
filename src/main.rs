use clap::Parser;
use futures::future;
use rustemon::pokemon::*;
use rustemon::Follow;

#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    #[arg(help="name of pokemon")]
    pokemon: String,

    //#[arg(short, help="recursively check evolution chain")]
    //recursive: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    println!("{}: {:?}", args.pokemon, get_types(&args).await);
}

async fn get_types(args: &Args) -> Vec<String> {
    // Create client
    let client = rustemon::client::RustemonClient::default();

    // Create pokemon resource
    let mon_resource = match pokemon::get_by_name(&args.pokemon, &client).await {
        Ok(x) => x,
        Err(_) => panic!("error: could not find pokemon {}", args.pokemon),
    };

    // Get types
    let types = match future::try_join_all(
        mon_resource.types.iter().map(async |t| t.type_.follow(&client).await)
    ).await {
        Ok(x) => x,
        Err(_) => panic!("error: could not retrive types for pokemon {}", args.pokemon),
    };

    // Return English names for types
    match future::try_join_all(types.into_iter().map(|t| t.names).map(
            async |names| {
                for n in names.iter() {
                    if n.language.follow(&client).await.unwrap().name == "en" {
                        return Ok(n.name.clone());
                    }
                }
                Err(())
            }
    )).await {
        Ok(x) => x,
        Err(_) => panic!("error: could not find English names for types"),
    }
}
