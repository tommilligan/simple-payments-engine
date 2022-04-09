pub mod csv;
pub mod store;
pub mod types;

use crate::{
    csv::{InputRow, OutputRow},
    store::Store,
    types::action::Action,
};
use log::warn;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("csv error")]
    Csv(#[from] ::csv::Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
}

pub fn run<R: std::io::Read, W: std::io::Write>(reader: R, writer: &mut W) -> Result<(), Error> {
    let mut csv_reader = ::csv::Reader::from_reader(reader);
    let mut csv_writer = ::csv::Writer::from_writer(writer);

    // Read in all the events and apply them
    let mut store = Store::default();
    for (index, result) in csv_reader.deserialize().into_iter().enumerate() {
        let row: InputRow = match result {
            Ok(row) => row,
            Err(error) => {
                match error.kind() {
                    // Otherwise
                    ::csv::ErrorKind::Deserialize { err: error, .. } => {
                        warn!("Failed deserializing action {index}: {error}");
                        continue;
                    }
                    _ => return Err(error.into()),
                }
            }
        };
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
            locked: client_state.is_locked(),
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn assert_run_produces(input: &str, expected: &str) {
        let mut actual = Vec::new();
        run(input.as_bytes(), &mut actual).unwrap();
        let actual = String::from_utf8(actual).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn spec_example() {
        let input = "type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0
";
        let expected = "client,available,held,total,locked
1,1.5,0.0,1.5,false
2,2.0,0.0,2.0,false
";
        assert_run_produces(input, expected);
    }

    #[test]
    fn duplicate_deposit() {
        let input = "type,client,tx,amount
deposit,1,1,1.0
deposit,1,1,1.0
";
        let expected = "client,available,held,total,locked
1,1.0,0.0,1.0,false
";
        assert_run_produces(input, expected);
    }

    #[test]
    fn duplicate_withdrawal() {
        let input = "type,client,tx,amount
deposit,1,2,10.0
withdrawal,1,1,1.0
withdrawal,1,1,1.0
";
        let expected = "client,available,held,total,locked
1,9.0,0.0,9.0,false
";
        assert_run_produces(input, expected);
    }

    #[test]
    fn duplicate_dispute() {
        let input = "type,client,tx,amount
deposit,1,1,1.0
deposit,1,2,2.0
dispute,1,2,
dispute,1,2,
";
        let expected = "client,available,held,total,locked
1,1.0,2.0,3.0,false
";
        assert_run_produces(input, expected);
    }

    #[test]
    fn duplicate_resolve() {
        let input = "type,client,tx,amount
deposit,1,1,1.0
deposit,1,2,2.0
dispute,1,2,
resolve,1,2,
resolve,1,2,
";
        let expected = "client,available,held,total,locked
1,3.0,0.0,3.0,false
";
        assert_run_produces(input, expected);
    }

    #[test]
    fn duplicate_chargeback() {
        let input = "type,client,tx,amount
deposit,1,1,1.0
deposit,1,2,2.0
dispute,1,2,
chargeback,1,2,
chargeback,1,2,
";
        let expected = "client,available,held,total,locked
1,1.0,0.0,1.0,true
";
        assert_run_produces(input, expected);
    }

    #[test]
    fn chargeback_negative() {
        let input = "type,client,tx,amount
deposit,1,2,2.0
withdrawal,1,3,1.0
dispute,1,2,
chargeback,1,2,
";
        let expected = "client,available,held,total,locked
1,-1.0,0.0,-1.0,true
";
        assert_run_produces(input, expected);
    }

    #[test]
    fn chargeback_freezes() {
        let input = "type,client,tx,amount
deposit,1,1,5.0
deposit,1,2,2.0
dispute,1,2,
chargeback,1,2,
withdrawal,1,3,3.0
";
        let expected = "client,available,held,total,locked
1,5.0,0.0,5.0,true
";
        assert_run_produces(input, expected);
    }

    #[test]
    fn dispute_after_resolve() {
        let input = "type,client,tx,amount
deposit,1,1,1.0
deposit,1,2,2.0
dispute,1,2,
resolve,1,2,
dispute,1,2,
";
        let expected = "client,available,held,total,locked
1,1.0,2.0,3.0,false
";
        assert_run_produces(input, expected);
    }

    #[test]
    fn dispute_after_chargeback() {
        let input = "type,client,tx,amount
deposit,1,1,1.0
deposit,1,2,2.0
dispute,1,2,
chargeback,1,2,
dispute,1,2,
";
        let expected = "client,available,held,total,locked
1,1.0,0.0,1.0,true
";
        assert_run_produces(input, expected);
    }

    #[test]
    fn client_with_no_successful_actions_in_output() {
        let input = "type,client,tx,amount
withdrawal,1,1,4.0
";
        let expected = "client,available,held,total,locked
1,0.0,0.0,0.0,false
";
        assert_run_produces(input, expected);
    }
}
