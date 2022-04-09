use anyhow::{anyhow, Context, Result};
use log::{error, info, warn};
use simple_payments_engine::{csv::InputRow, store::Store, types::action::Action};
use std::env::args;
use std::fs::File;
use std::io::BufReader;

fn run() -> Result<()> {
    let args: Vec<String> = args().collect();
    let input_filepath = args
        .get(1)
        .context(anyhow!("Missing input file as first positional argument."))?;

    info!("Reading data from {:?}", input_filepath);
    let input_reader = BufReader::new(File::open(input_filepath)?);
    let mut csv_reader = csv::Reader::from_reader(input_reader);

    let mut store = Store::default();
    for (index, result) in csv_reader.deserialize().into_iter().enumerate() {
        let row: InputRow = result?;
        let action: Action = match row.try_into() {
            Ok(action) => action,
            Err(error) => {
                warn!("Action {index} invalid: {error}");
                continue;
            }
        };
        if let Err(error) = store.apply(action) {
            warn!("Action {index} not applied: {error}");
        }
    }
    Ok(())
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    match run() {
        Ok(()) => {}
        Err(error) => {
            error!("{}", error);
            std::process::exit(1)
        }
    }
}
