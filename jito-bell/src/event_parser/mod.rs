use crate::event_parser::jito_steward::JitoStewardEvent;

pub mod jito_steward;

#[derive(Debug)]
pub enum EventParser {
    JitoSteward(JitoStewardEvent),
}
