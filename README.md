# pokelookup [![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/pokelookup.svg
[crates.io]: https://crates.io/crates/pokelookup

This is a package that gives an executable to help look up Pokemon details using [PokeAPI](https://pokeapi.co/) using the 
[rustemon](https://crates.io/crates/rustemon) wrapper. The command allows for quick look-up of types, abilities, egg groups, and more.

## Usage

After installing the package, run `pokelookup --help` to see all possible subcommands and options. Note that since this package uses 
PokeAPI to get its information, Pokemon will need qualifiers if there are multiple forms. For example, to look up the types for Toxtricity, 
you will need to specify which form (Amped or Low-Key). The `pokelookup list` subcommand is a way to look up varieties of a given Pokemon
species for help finding the needed identifier.

## Contributing
Contributions and feedback is welcome! Feel free to open a PR or add an issue in the Issues tab.
