pub mod csv;
pub mod store;
pub mod types;

use crate::{
    csv::{InputRow, OutputRow},
    store::{client::Access, Store},
    types::action::Action,
};
use log::warn;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("csv error")]
    CsvError(#[from] ::csv::Error),
}

pub fn run<R: std::io::Read, W: std::io::Write>(reader: R, writer: &mut W) -> Result<(), Error> {
    let mut csv_reader = ::csv::Reader::from_reader(reader);
    let mut csv_writer = ::csv::Writer::from_writer(writer);

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
    for (client_id, client_state) in clients.into_iter() {
        csv_writer.serialize(OutputRow {
            client: client_id.0,
            available: client_state.available(),
            held: client_state.held,
            total: client_state.total,
            locked: client_state.access == Access::Frozen,
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
