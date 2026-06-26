use crate::model::Tick;
use crate::tdx_reader::{read_ticks_from_tdx_text, TdxReadOptions};
use crate::text_reader::{read_ticks_from_text, TickTextReadOptions};
use anyhow::{anyhow, Result};
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
    let is_txt = path
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.eq_ignore_ascii_case("txt"))
        .unwrap_or(false);

    if is_txt {
        if let Ok(ticks) = read_ticks_from_tdx_text(path, &tdx_options(options)) {
            if !ticks.is_empty() {
                return Ok(ticks);
            }
        }
    }

    let text_result = read_ticks_from_text(path, &text_options(options));
    match text_result {
        Ok(ticks) => Ok(ticks),
        Err(text_err) => match read_ticks_from_tdx_text(path, &tdx_options(options)) {
            Ok(ticks) => Ok(ticks),
            Err(tdx_err) => Err(anyhow!(
                "text parser failed: {text_err}; tdx parser failed: {tdx_err}"
            )),
        },
    }
}

fn text_options(options: &TickCsvReadOptions) -> TickTextReadOptions {
    TickTextReadOptions {
        default_symbol: options.default_symbol.clone(),
        price_scale: options.price_scale,
    }
}

fn tdx_options(options: &TickCsvReadOptions) -> TdxReadOptions {
    TdxReadOptions {
        default_symbol: options.default_symbol.clone(),
        price_scale: options.price_scale,
    }
}
