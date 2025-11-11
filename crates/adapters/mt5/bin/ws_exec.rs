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
// See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! WebSocket server for MT5 execution streaming.

use std::net::SocketAddr;

use axum::{response::Json, routing::get, Router, serve};
use serde_json::json;
use tracing::info;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting MT5 WebSocket Execution server...");

    // Define the address to bind to
    let addr: SocketAddr = ([127, 0, 0, 1], 3002).into();

    // Build the application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health));

    // Run the server
    info!("MT5 WebSocket Execution server running on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "service": "nautilus-adapters-mt5-ws-exec",
        "status": "running",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}