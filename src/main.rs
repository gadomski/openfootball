extern crate clap;
extern crate failure;
extern crate openfootball;
extern crate serde;
extern crate serde_json;

use clap::{App, Arg};
use failure::Error;
use openfootball::Season;

fn main() -> Result<(), Error> {
    let matches = App::new("openfootball")
        .arg(
            Arg::with_name("INFILE")
                .help("Sets the input openfootball text file")
                .required(true)
                .index(1),
        )
        .get_matches();
    let infile = matches.value_of("INFILE").unwrap();
    let season = Season::from_path(infile)?;
    let standings = season.standings();
    println!("{}", serde_json::to_string_pretty(&standings)?);
    Ok(())
}
