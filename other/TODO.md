# MT5 Adapter Refactoring & Roadmap

Based on project analysis and NautilusTrader best practices.

## 1. Rust Core Refactoring (High Priority)
The goal is to move domain logic and data conversion from Python to the Rust core for performance and correctness, following standard adapter patterns (BitMEX, OKX).

- [x] **Dependencies**: `Cargo.toml` uses `workspace = true` for common crates (`nautilus-model`, `chrono`, etc.). ✅
- [x] **Implement `request_bars` in Rust**: ✅
    - Updated `Mt5DataClient` in `src/data_client.rs` with `py_request_bars`.
    - **Parsing**: `parse_bar_row` implemented with proper docs (TODO: move to `src/http/parse.rs`).
    - Conversion from MT5 JSON -> `Vec<Bar>` using `UnixNanos`, `Price::from_f64`, `Quantity::from_f64`.
    - **Logging**: Uses `tracing::error!` in pagination loop.
    - **Error Handling**: MT5 errors mapped to `PyRuntimeError`.
    - PyO3 bindings expose `request_bars` returning `List[Bar]`.
- [x] **Implement `BarType` parsing**: Uses `bar_type.spec().aggregation()` and `bar_type.spec().step()`. ✅

## 2. Python Layer Cleanup
Once the Rust core handles the heavy lifting, the Python layer should become thinner.

- [ ] **Update `Mt5Client` (Python)**:
    - Deprecate/Remove the Python-side `request_bars` loop once Rust version is ready.
    - Update calls to use the new Rust method.
- [ ] **Refactor `execution.py`**:
    - Deduplicate `_generate_order_*` methods (create shared helpers).
    - Centralize order type conversion logic.
    - Consolidate report parsing (position, order status, fill reports).

## 3. Feature Completion (Missing Functionality)
Address gaps identified in `RECAP.md`.

- [ ] **Trade Tick Polling**: Implement `_poll_trades` in `data.py` (currently a placeholder).
- [ ] **Advanced Order Types**:
    - Implement and test Limit Orders.
    - Implement and test Stop Orders.
    - Implement SL/TP extraction for bracket orders.
- [ ] **Position Reconciliation**: Ensure local cache matches venue state on start/reconnect.

## 4. Testing & Validation (Standard Compliance)
- [ ] **Test Data**: Add canonical JSON response files in `crates/adapters/mt5/test_data/` (e.g., `http_get_bars.json`) for unit tests.
- [ ] **Rust Unit Tests**: Add `#[cfg(test)]` mod in `data_client.rs` to test `parse_bar_row` using the JSON test data.
- [ ] **Integration Tests**:
    - `tests/test_backtest.py`: Verify no regression with new Rust path.
    - `test_orders.py`: Verify limit/stop order behavior.
- [ ] **Type Stubs**: Update `.pyi` files to reflect `request_bars` returning `list[Bar]`.

## Recommended Workflow
1. **Rust**: detailed implementation of `request_bars` (requires adding deps).
2. **Python**: Switch `client.py` to use the new Rust method.
3. **Verify**: Run `test_backtest.py` to ensure no regression.
4. **Cleanup**: Refactor `execution.py`.
