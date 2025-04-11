use std::collections::HashMap;

use serde::Deserialize;

use crate::program::Program;

#[derive(Deserialize)]
pub struct Config {
    pub programs: HashMap<String, Program>,
}
