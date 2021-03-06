extern crate clap;
extern crate csv;
extern crate failure;
extern crate openfootball;

use failure::Error;

fn main() -> Result<(), Error> {
    use clap::{App, Arg, SubCommand};
    use csv::Writer;
    use openfootball::Season;
    use std::io;

    let infile = Arg::with_name("INFILE")
        .help("Sets the input openfootball text file")
        .required(true)
        .index(1);
    let matches = App::new("openfootball")
        .subcommand(
            SubCommand::with_name("standings")
                .about("Prints standings as CSV data")
                .arg(infile.clone()),
        )
        .subcommand(
            SubCommand::with_name("odds")
                .about("Prints odds for a given matchweek")
                .arg(infile)
                .arg(
                    Arg::with_name("MATCHWEEK")
                        .help("The matchweek")
                        .required(true)
                        .index(2),
                ),
        )
        .get_matches();
    let mut writer = Writer::from_writer(io::stdout());
    if let Some(matches) = matches.subcommand_matches("standings") {
        let season = Season::from_path(matches.value_of("INFILE").unwrap())?;
        for standing in season.standings(1500, 32.)? {
            writer.serialize(standing)?;
        }
    } else if let Some(matches) = matches.subcommand_matches("odds") {
        let season = Season::from_path(matches.value_of("INFILE").unwrap())?;
        for odds in season.odds(1500, 32., matches.value_of("MATCHWEEK").unwrap().parse()?)? {
            writer.serialize(odds)?;
        }
    }
    Ok(())
}
