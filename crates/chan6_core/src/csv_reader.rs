use crate::model::Tick;
use crate::tdx_reader::{read_ticks_from_tdx_text, TdxReadOptions};
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
    let text_result = read_ticks_from_text(
        path,
        &TickTextReadOptions {
            default_symbol: options.default_symbol.clone(),
            price_scale: options.price_scale,
        },
    );

    match text_result {
        Ok(ticks) => Ok(ticks),
        Err(text_err) => match read_ticks_from_tdx_text(
            path,
            &TdxReadOptions {
                default_symbol: options.default_symbol.clone(),
                price_scale: options.price_scale,
            },
        ) {
            Ok(ticks) => Ok(ticks),
            Err(tdx_err) => Err(text_err.context(format!("tdx fallback also failed: {tdx_err}"))),
        },
    }
}
