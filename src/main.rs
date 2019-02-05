extern crate clap;
extern crate csv;
extern crate failure;
extern crate openfootball;

use clap::{App, Arg};
use csv::Writer;
use failure::Error;
use openfootball::Season;
use std::io;

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
    let teams = season.teams();
    let mut writer = Writer::from_writer(io::stdout());
    let mut header = vec!["Matchday".to_string()];
    header.extend(teams.clone());
    writer.write_record(header)?;
    for (matchday, ratings) in season.ratings() {
        let mut row = vec![format!("{}", matchday)];
        for team in &teams {
            row.push(format!("{}", ratings[team]));
        }
        writer.write_record(row)?;
    }
    Ok(())
}
