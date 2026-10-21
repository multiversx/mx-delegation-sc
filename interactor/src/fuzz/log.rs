//! The per-event action log.
//!
//! The chain simulator does not reproduce from a seed alone: `generate_blocks_until_tx_
//! processed` produces a variable number of blocks, so block nonces - and with them
//! unbond timing - drift between two runs of the same seed. The log records what actually
//! happened, parameters and all, which is the dependable way back to a failing run.

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::fuzz::Event;

pub struct ActionLog {
    writer: BufWriter<File>,
}

impl ActionLog {
    pub fn create(path: &Path, seed: u64, iterations: usize) -> Self {
        let file = File::create(path)
            .unwrap_or_else(|e| panic!("cannot create the action log at {}: {e}", path.display()));

        let mut log = ActionLog {
            writer: BufWriter::new(file),
        };
        log.write_line(serde_json::json!({
            "seed": seed,
            "iterations": iterations,
        }));

        log
    }

    pub fn write(&mut self, step: usize, event: &Event) {
        let mut value = event.log_value();
        value["step"] = serde_json::json!(step);
        self.write_line(value);
    }

    fn write_line(&mut self, value: serde_json::Value) {
        writeln!(self.writer, "{value}").expect("cannot write to the action log");
        // Flushed per line: a run that ends in a panic must still leave a usable log.
        self.writer.flush().expect("cannot flush the action log");
    }
}
