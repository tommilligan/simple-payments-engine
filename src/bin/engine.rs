use anyhow::{anyhow, Context, Result};
use log::{error, info, warn};
use simple_payments_engine::{
    csv::{InputRow, OutputRow},
    store::{client::Access, Store},
    types::action::Action,
};
use std::env::args;
use std::fs::File;
use std::io::{stdout, BufReader};

fn run() -> Result<()> {
    let args: Vec<String> = args().collect();
    let input_filepath = args
        .get(1)
        .context(anyhow!("Missing input file as first positional argument."))?;

    info!("Reading data from {:?}", input_filepath);
    let input_reader = BufReader::new(File::open(input_filepath)?);
    let mut csv_reader = csv::Reader::from_reader(input_reader);

    // Read in all the events and apply them
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

    // Return output
    let clients = store.into_client_store();
    let stdout = stdout();
    let mut writer = csv::Writer::from_writer(stdout);
    for (client_id, client_state) in clients.into_iter() {
        writer.serialize(OutputRow {
            client: client_id.0,
            available: client_state.available(),
            held: client_state.held,
            total: client_state.total,
            locked: client_state.access == Access::Frozen,
        })?;
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
