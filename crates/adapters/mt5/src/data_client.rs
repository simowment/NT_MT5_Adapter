// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------
//
//! MT5 Data Client implementation.
//!
//! This module implements the data client for the MetaTrader 5 adapter,
//! providing market data functionality including subscriptions and requests.

use crate::config::{Mt5Config, Mt5DataClientConfig};
use crate::http::client::Mt5HttpClient;
use crate::http::error::Mt5HttpError as HttpClientError;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] HttpClientError),
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl From<String> for DataClientError {
    fn from(s: String) -> Self {
        DataClientError::ParseError(s)
    }
}

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;
#[cfg(feature = "python-bindings")]
use nautilus_core::nanos::UnixNanos;
#[cfg(feature = "python-bindings")]
use nautilus_model::{
    data::{Bar, BarType},
    enums::BarAggregation,
    types::{Price, Quantity},
};
#[cfg(feature = "python-bindings")]
use chrono::{DateTime, Utc};

#[cfg(feature = "python-bindings")]
#[derive(Clone, Debug)]
#[pyclass]
pub struct Mt5DataClient {
    #[pyo3(get)]
    config: Mt5DataClientConfig,
    http_client: Arc<Mt5HttpClient>,
}

#[cfg(not(feature = "python-bindings"))]
pub struct Mt5DataClient {
    pub config: Mt5DataClientConfig,
    http_client: Arc<Mt5HttpClient>,
}

impl Mt5DataClient {
    /// Creates a new instance of the MT5 data client.
    pub fn new(config: Mt5DataClientConfig) -> Result<Self, DataClientError> {
        let base_url = config.base_url.clone();
        let http_config = Mt5Config {
            base_url: base_url.clone(),
            http_timeout: config.http_timeout,
            proxy: None,
        };

        let http_client = Arc::new(Mt5HttpClient::new(http_config, base_url).map_err(|e| DataClientError::ConnectionError(e.to_string()))?);

        Ok(Self { config, http_client })
    }

    /// Performs a login to validate connectivity with the MT5 bridge.
    pub async fn connect(&self) -> Result<(), DataClientError> {
        self.http_client.login().await.map_err(|e| DataClientError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    /// Fetches all symbols from the MT5 bridge.
    pub async fn get_symbols(&self) -> Result<Vec<crate::http::models::Mt5Symbol>, DataClientError> {
        let body = serde_json::json!({});
        let response = self.http_client.symbols_get(&body).await.map_err(|e| DataClientError::ConnectionError(e.to_string()))?;
        let symbols: Vec<crate::http::models::Mt5Symbol> = serde_json::from_value(response)
            .map_err(|e| DataClientError::ParseError(e.to_string()))?;
        Ok(symbols)
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5DataClient {
    #[new]
    pub fn py_new(config: Mt5DataClientConfig) -> Result<Self, PyErr> {
        Self::new(config).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    #[pyo3(name = "connect")]
    pub fn py_connect<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client.connect().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    #[pyo3(name = "get_symbols")]
    pub fn py_get_symbols<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let symbols = client.get_symbols().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            // Return as JSON string
            serde_json::to_string(&symbols)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Fetches historical bars for a specific instrument.
    #[pyo3(name = "fetch_bars")]
    pub fn py_fetch_bars<'py>(&self, py: Python<'py>, symbol: String, timeframe: u32, start_time: i64, count: u32) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            // Construct the request array expected by the MT5 middleware [symbol, timeframe, start_time, count]
            let body = serde_json::json!([symbol, timeframe, start_time, count]);
            
            // Use the underlying http_client to send the request
            let result = client.http_client.copy_rates_from(&body).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            // Check for error field in response
            if let Some(error) = result.get("error") {
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(error.to_string()));
            }
                
            // Extract the result list
            let bars_value = result.get("result")
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("No result field in response"))?;

            // Deserialize into Vec of tuples/arrays to avoid string parsing in Python
            // MT5 returns: [time, open, high, low, close, tick_volume, spread, real_volume]
            let bars_raw: Vec<(i64, f64, f64, f64, f64, u64, i32, u64)> = serde_json::from_value(bars_value.clone())
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to parse bars: {}", e)))?;

            // We need to return a Python object. Since we are in an async block that returns a Result<T>,
            // and T must directly convert to Python object, we can construct the list of dicts here?
            // No, we cannot access Python GIL here easily to create PyDicts if we are in a separate thread.
            // But `future_into_py` handles the return value conversion ON THE MAIN THREAD (with GIL).
            // So we can return a struct that implements IntoPy<PyObject>.
            
            Ok(Mt5BarList(bars_raw))
        })
    }

    /// Requests historical bars and returns Nautilus Bar objects (Paginated).
    #[pyo3(name = "request_bars")]
    #[allow(clippy::too_many_arguments)]
    pub fn py_request_bars<'py>(
        &self,
        py: Python<'py>,
        bar_type: BarType,
        instrument: Bound<'py, PyAny>,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        count: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        
        // Extract symbol and precisions
        let symbol = bar_type.instrument_id().symbol().as_str().to_string();
        let price_precision: u8 = instrument.getattr("price_precision")?.extract()?;
        let size_precision: u8 = instrument.getattr("size_precision")?.extract()?;
        let bar_type_clone = bar_type.clone();

        // Calculate timeframe in seconds for bar close calculation
        let tf_seconds = match bar_type.spec().aggregation() {
            BarAggregation::Second => bar_type.spec().step(),
            BarAggregation::Minute => bar_type.spec().step() * 60,
            BarAggregation::Hour => bar_type.spec().step() * 3600,
            BarAggregation::Day => bar_type.spec().step() * 86400,
            _ => 60, // Default to 1 minute
        };

        // Determine MT5 timeframe value
        let mt5_tf = match tf_seconds {
            60 => 1,
            300 => 5,
            900 => 15,
            1800 => 30,
            3600 => 16385, // H1
            14400 => 16388, // H4
            86400 => 16408, // D1
            _ => 1, // Fallback M1
        };

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut bars: Vec<Bar> = Vec::new();
            
            // Logic:
            // 1. If start and end provided -> Range Request (Loop)
            // 2. Else -> Count Request from now
            
            if let (Some(start_dt), Some(end_dt)) = (start, end) {
                let mut current_start = start_dt.timestamp();
                let end_ts = end_dt.timestamp();
                let chunk_size = 30 * 24 * 3600; // 30 days chunk
                
                while current_start < end_ts {
                    let current_end = std::cmp::min(current_start + chunk_size, end_ts);
                     // [symbol, timeframe, start, end]
                    let body = serde_json::json!([symbol, mt5_tf, current_start, current_end]);
                    
                    match client.http_client.copy_rates_range(&body).await {
                        Ok(val) => {
                             if let Some(res) = val.get("result") {
                                 if let Ok(rows) = serde_json::from_value::<Vec<Vec<serde_json::Value>>>(res.clone()) {
                                    for row in rows {
                                        if let Some(bar) = parse_bar_row(&row, &bar_type_clone, tf_seconds, price_precision, size_precision) {
                                            bars.push(bar);
                                        }
                                    }
                                 }
                             }
                        },
                        Err(e) => tracing::error!("Error fetching bars chunk: {}", e),
                    }
                    
                    current_start = current_end;
                    // Yield to avoid blocking executor too long (optional)
                    tokio::task::yield_now().await; 
                }
            } else {
                // Count request
                let count_val = count.unwrap_or(1000);
                let now = Utc::now().timestamp();
                // [symbol, timeframe, start, count]
                let body = serde_json::json!([symbol, mt5_tf, now, count_val]);
                
                 let result = client.http_client.copy_rates_from(&body).await
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
                
                 if let Some(res) = result.get("result") {
                     let rows: Vec<Vec<serde_json::Value>> = serde_json::from_value(res.clone())
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
                     
                     for row in rows {
                        if let Some(bar) = parse_bar_row(&row, &bar_type_clone, tf_seconds, price_precision, size_precision) {
                            bars.push(bar);
                        }
                     }
                 }
            }

            Python::attach(|py| {
                let py_bars: PyResult<Vec<_>> = bars.into_iter().map(|bar| bar.into_py_any(py)).collect();
                Ok(py_bars?)
            })
        })
    }
}

// Helper struct to handle conversion to Python List[Dict]
struct Mt5BarList(Vec<(i64, f64, f64, f64, f64, u64, i32, u64)>);

impl<'py> IntoPyObject<'py> for Mt5BarList {
    type Target = pyo3::types::PyList;
    type Output = pyo3::Bound<'py, Self::Target>;
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let list = pyo3::types::PyList::empty(py);
        for (time, open, high, low, close, tick_vol, spread, real_vol) in self.0 {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("time", time)?;
            dict.set_item("open", open)?;
            dict.set_item("high", high)?;
            dict.set_item("low", low)?;
            dict.set_item("close", close)?;
            dict.set_item("tick_volume", tick_vol)?;
            dict.set_item("spread", spread)?;
            dict.set_item("real_volume", real_vol)?;
            list.append(dict)?;
        }
        Ok(list)
    }
}

/// Parses a raw MT5 bar row into a Nautilus `Bar` object.
///
/// # Arguments
///
/// * `row` - Raw JSON array [time, open, high, low, close, tick_vol, spread, real_vol]
/// * `bar_type` - The bar type specification
/// * `tf_seconds` - Timeframe in seconds for calculating bar close time
/// * `price_prec` - Price precision from instrument
/// * `size_prec` - Size precision from instrument
///
/// # Returns
///
/// Returns `Some(Bar)` on success, `None` if parsing fails.
#[cfg(feature = "python-bindings")]
fn parse_bar_row(
    row: &[serde_json::Value],
    bar_type: &BarType,
    tf_seconds: u64,
    price_prec: u8,
    size_prec: u8,
) -> Option<Bar> {
    if row.len() < 6 {
        return None;
    }

    let time_sec = row[0].as_i64()?;
    let open = row[1].as_f64()?;
    let high = row[2].as_f64()?;
    let low = row[3].as_f64()?;
    let close = row[4].as_f64()?;
    let tick_vol = row[5].as_f64()?;

    let ts_open = time_sec as u64;
    let ts_init_ns = UnixNanos::from((ts_open + tf_seconds) * 1_000_000_000);

    Some(Bar::new(
        bar_type.clone(),
        Price::from_f64(open, price_prec).ok()?,
        Price::from_f64(high, price_prec).ok()?,
        Price::from_f64(low, price_prec).ok()?,
        Price::from_f64(close, price_prec).ok()?,
        Quantity::from_f64(tick_vol, size_prec).ok()?,
        ts_init_ns,
        ts_init_ns,
    ))
}