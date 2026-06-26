use crate::model::Tick;
use crate::text_reader::{read_ticks_from_text, TickTextReadOptions};
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TickCsvReadOptions {
    pub default_symbol: Option<String>,
    pub price_scale: f64,
}

impl Default for TickCsvReadOptions {
    fn default() -> Self {
        Self {
            default_symbol: None,
            price_scale: 1000.0,
        }
    }
}

pub fn read_ticks_from_csv(path: &Path, options: &TickCsvReadOptions) -> Result<Vec<Tick>> {
    read_ticks_from_text(
        path,
        &TickTextReadOptions {
            default_symbol: options.default_symbol.clone(),
            price_scale: options.price_scale,
        },
    )
}
