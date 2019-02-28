extern crate clap;
extern crate csv;
extern crate failure;
extern crate openfootball;

use failure::Error;

fn main() -> Result<(), Error> {
    use clap::{App, Arg};
    use csv::Writer;
    use openfootball::Season;
    use std::io;

    let matches = App::new("openfootball")
        .arg(
            Arg::with_name("INFILE")
                .help("Sets the input openfootball text file")
                .required(true)
                .index(1),
        )
        .get_matches();
    let season = Season::from_path(matches.value_of("INFILE").unwrap())?;
    let mut writer = Writer::from_writer(io::stdout());
    for game in season.games() {
        writer.serialize(game)?;
    }
    Ok(())
}
