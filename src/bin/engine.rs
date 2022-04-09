use anyhow::{anyhow, Context, Result};
use log::{error, info, warn};
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
    if let Err(error) = simple_payments_engine::run(reader, &mut writer) {
        warn!("error processing csv data: {}", error);
    };
    Ok(())
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    if let Err(error) = run() {
        error!("{}", error);
        std::process::exit(1)
    }
}
