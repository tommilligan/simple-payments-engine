use anyhow::{anyhow, Context, Result};
use log::{error, info};
use std::env::args;
use std::fs::File;
use std::io::{stdout, BufReader};

fn run() -> Result<()> {
    let args: Vec<String> = args().collect();
    let input_filepath = args
        .get(1)
        .context(anyhow!("Missing input file as first positional argument."))?;

    info!("Reading data from {:?}", input_filepath);
    let reader = BufReader::new(File::open(input_filepath)?);
    let mut writer = stdout();
    simple_payments_engine::run(reader, &mut writer)?;

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
