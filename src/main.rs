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
    #[command(name="types")]
    TypeCmd {
        #[arg(help="name of pokemon")]
        pokemon: String,

        //#[arg(short, help="recursively check evolution chain")]
        //recursive: bool,
    },

    #[command(name="abilities")]
    AbilityCmd {
        #[arg(help="name of pokemon")]
        pokemon: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    match args.command {
        SubArgs::TypeCmd{..} => {
            print_types(&args.command).await;
        },
        _ => panic!("error: not yet implemented"),
    };
}

async fn print_types(args: &SubArgs) {
    let SubArgs::TypeCmd{pokemon} = args else {
        panic!("error: incorrect inputs");
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

    // Return English names for types
    let result = match future::try_join_all(types.into_iter().map(|t| t.names).map(
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
    };

    println!("{}: {:?}", pokemon, result);
}
