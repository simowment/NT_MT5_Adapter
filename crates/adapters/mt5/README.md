# Nautilus Trader MetaTrader 5 Adapter

Un adaptateur Rust complet pour intégrer MetaTrader 5 (MT5) avec Nautilus Trader, fournissant les APIs HTTP REST et WebSocket pour le trading en temps réel et la gestion d'ordres.

## 🎯 Fonctionnalités

### **Architecture Complète**
- **Client HTTP MT5** : API REST avec authentification, retry, et taxonomie d'erreurs
- **Client WebSocket MT5** : Streaming temps réel avec gestion d'état et reconnexion
- **Mt5InstrumentProvider** : Découverte automatique d'instruments avec cache intelligent
- **Mt5DataClient** : Souscriptions quotes/trades/bars, requêtes historiques
- **Mt5ExecutionClient** : Gestion complète d'ordres (submit, modify, cancel) avec reports

### **Fonctionnalités Avancées**
- **Configuration enrichie** : Configurations séparées pour Instrument/Data/Execution
- **Gestion d'erreurs sophistiquée** : Taxonomie complète (retryable/non-retryable/fatal)
- **Logging structuré** : Traçabilité et debugging avec exceptions Python
- **Bindings Python PyO3** : Intégration fluide avec l'écosystème Nautilus
- **Parseurs robustes** : Conversion MT5 → modèles Nautilus (FX, CFD, Futures)

## 🏗️ Architecture

L'adaptateur suit une architecture modulaire en couches :

```
crates/adapters/mt5/
├── src/
│   ├── common/                    # Types partagés et utilitaires
│   │   ├── consts.rs             # Constantes MT5
│   │   ├── credential.rs         # Gestion des identifiants MT5
│   │   ├── enums.rs              # Énumérations MT5
│   │   ├── urls.rs               # Résolution d'URLs
│   │   ├── parse.rs              # Parseurs communs (instruments, timestamps, prix)
│   │   └── testing.rs            # Fixtures pour tests
│   ├── config/                   # Configurations enrichies
│   │   ├── instrument_provider.rs
│   │   ├── data_client.rs
│   │   └── execution_client.rs
│   ├── http/                     # Client HTTP avec pattern inner/outer
│   │   ├── client.rs             # Mt5HttpClient (clonable, thread-safe)
│   │   ├── rest_client.rs        # Mt5HttpInnerClient (état privé)
│   │   ├── models.rs             # Structs REST (AccountInfo, Symbol, Rate, etc.)
│   │   ├── query.rs              # Query builders avec derive_builder
│   │   └── parse.rs              # Parsing réponses HTTP → Nautilus
│   ├── websocket/                # Client WebSocket temps réel
│   │   ├── client.rs             # Mt5WebSocketClient avec gestion d'état
│   │   ├── messages.rs           # Structs WebSocket (quotes, trades, order book)
│   │   └── parse.rs              # Parsing messages WebSocket
│   ├── python/                   # Bindings PyO3
│   │   ├── bindings.rs           # Exposition classes Rust → Python
│   │   └── mod.rs                # Module Python
│   ├── instrument_provider.rs    # Provider d'instruments
│   ├── data_client.rs            # Client de données temps réel
│   ├── execution_client.rs       # Client d'exécution d'ordres
│   └── lib.rs                    # Point d'entrée de la bibliothèque
└── tests/                        # Tests unitaires et d'intégration
    └── integration_tests.rs
```

### **Flux de Données**

```
MT5 Server ──→ HTTP Client ──→ Rust Models ──→ Python Layer ──→ Nautilus Trader
                │
                └──→ WebSocket Client ──→ Real-time Events ──→ MessageBus
```

## 🚀 Installation et Compilation

### **Compilation Rust**

```bash
# Compilation de base
cargo build -p nautilus-adapters-mt5

# Avec bindings Python
cargo build -p nautilus-adapters-mt5 --features python-bindings

# Build optimisé pour production
cargo build -p nautilus-adapters-mt5 --release --features python-bindings
```

### **Package Python**

```bash
# Génération du package wheel
maturin build --release --features python-bindings

# Installation du package
pip install target/wheels/*.whl
```

### **Script de Compilation**

```bash
# Linux/Mac
./build_mt5_adapter.sh

# Windows
build_mt5_adapter.bat
```

## 🔧 Configuration

### **Configuration Instrument Provider**

```python
from nautilus_trader.adapters.mt5.config import Mt5InstrumentProviderConfig

config = Mt5InstrumentProviderConfig(
    mt5_host="localhost",
    mt5_port=8080,
    mt5_login="your_login",
    mt5_password="your_password",
    mt5_server="your_server",
    filter_currencies=["USD", "EUR"],
    filter_indices=["US30", "SPX500"],
    filter_cfds=True,
    filter_futures=False,
    auto_discover_instruments=True,
    cache_expiry=300,
    enable_logging=True
)
```

### **Configuration Data Client**

```python
from nautilus_trader.adapters.mt5.config import Mt5DataClientConfig

config = Mt5DataClientConfig(
    mt5_host="localhost",
    mt5_port=8080,
    mt5_login="your_login",
    mt5_password="your_password",
    mt5_server="your_server",
    subscribe_quotes=True,
    subscribe_trades=True,
    subscribe_order_book=False,
    max_subscriptions=1000,
    connection_retry_attempts=3,
    connection_retry_delay=5,
    heartbeat_interval=30,
    reconnection_enabled=True,
    enable_logging=True
)
```

### **Configuration Execution Client**

```python
from nautilus_trader.adapters.mt5.config import Mt5ExecClientConfig

config = Mt5ExecClientConfig(
    mt5_host="localhost",
    mt5_port=8080,
    mt5_login="your_login",
    mt5_password="your_password",
    mt5_server="your_server",
    max_concurrent_orders=50,
    order_timeout=30,
    connection_retry_attempts=3,
    connection_retry_delay=5,
    enable_partial_fills=True,
    enable_market_data=True,
    risk_management_enabled=True,
    position_sizing_enabled=True,
    simulate_orders=False,
    enable_logging=True
)
```

## 💻 Utilisation

### **Exemple Rust - Client HTTP**

```rust
use nautilus_adapters_mt5::{Mt5HttpClient, Mt5Config, Mt5Credential};
use nautilus_adapters_mt5::common::urls::Mt5Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration
    let config = Mt5Config {
        api_key: "your_api_key".to_string(),
        api_secret: "your_api_secret".to_string(),
        base_url: "http://localhost:8080".to_string(),
        ws_url: "ws://localhost:8080".to_string(),
        http_timeout: 30,
        ws_timeout: 30,
        proxy: None,
    };

    let credential = Mt5Credential {
        login: "your_login".to_string(),
        password: "your_password".to_string(),
        server: "your_server".to_string(),
        proxy: None,
        token: None,
    };

    let url = Mt5Url::new("http://localhost:8080");
    
    // Création du client
    let client = Mt5HttpClient::new(config, credential, url)?;
    
    // Connexion et authentification
    client.login().await?;
    
    // Récupération des symboles
    let symbols = client.get_symbols().await?;
    println!("Symboles disponibles: {}", symbols.len());
    
    // Récupération des données historiques
    let rates = client.get_rates("EURUSD").await?;
    println!("Rates EURUSD: {}", rates.len());
    
    Ok(())
}
```

### **Exemple Python - Data Client**

```python
import asyncio
from nautilus_trader.adapters.mt5 import (
    Mt5HttpClient, Mt5WebSocketClient, Mt5DataClientConfig
)
from nautilus_trader.common.component import MessageBus, Cache
from nautilus_trader.common.component import LiveClock

async def main():
    # Configuration
    config = Mt5DataClientConfig(
        mt5_host="localhost",
        mt5_port=8080,
        mt5_login="your_login",
        mt5_password="your_password",
        mt5_server="your_server"
    )
    
    # Création des clients
    http_client = Mt5HttpClient(config, "your_login", "your_password", "your_server")
    ws_client = Mt5WebSocketClient("your_login", "your_password", "your_server", "ws://localhost:8080")
    
    # Connexion
    http_client.login()
    ws_client.connect()
    ws_client.authenticate()
    
    # Souscriptions
    await ws_client.subscribe_quotes("EURUSD")
    await ws_client.subscribe_trades("EURUSD")
    
    # Requêtes historiques
    rates = await http_client.get_rates("EURUSD")
    print(f"Reçu {len(rates)} rates pour EURUSD")
    
    # Connexion au système Nautilus
    data_client = Mt5DataClient(
        loop=asyncio.get_event_loop(),
        http_client=http_client,
        ws_client=ws_client,
        msgbus=MessageBus(),
        cache=Cache(),
        clock=LiveClock()
    )
    
    # Connexion
    data_client.connect()
    
    # Souscription à des données
    from nautilus_trader.model.identifiers import InstrumentId
    instrument_id = InstrumentId.from_str("EURUSD.MT5")
    await data_client._subscribe_quote_ticks(instrument_id)
    
    print("Data client opérationnel!")

if __name__ == "__main__":
    asyncio.run(main())
```

### **Exemple Python - Execution Client**

```python
import asyncio
from nautilus_trader.execution.messages import SubmitOrder
from nautilus_trader.model.identifiers import InstrumentId, ClientOrderId
from nautilus_trader.model.objects import OrderSide, OrderType, Quantity

async def main():
    # Configuration et clients (même setup que data client)
    # ...
    
    # Créer un client d'exécution
    exec_client = Mt5ExecutionClient(
        loop=asyncio.get_event_loop(),
        http_client=http_client,
        ws_client=ws_client,
        msgbus=MessageBus(),
        cache=Cache(),
        clock=LiveClock()
    )
    
    # Connexion
    exec_client.connect()
    
    # Soumettre un ordre
    order = SubmitOrder(
        instrument_id=InstrumentId.from_str("EURUSD.MT5"),
        client_order_id=ClientOrderId("test_order_001"),
        order_side=OrderSide.BUY,
        order_type=OrderType.MARKET,
        quantity=Quantity.from_str("0.1"),
    )
    
    await exec_client._submit_order(order)
    print("Ordre soumis avec succès!")

if __name__ == "__main__":
    asyncio.run(main())
```

## 🧪 Tests

### **Tests Rust**

```bash
# Tests unitaires
cargo test -p nautilus-adapters-mt5

# Tests avec mocks
cargo test -p nautilus-adapters-mt5 --test integration_tests

# Tests avec coverage
cargo test -p nautilus-adapters-mt5 -- --cov
```

### **Test de Compilation**

```bash
# Valider la structure des configurations
rustc simple_test.rs && ./simple_test.exe
```

### **Backtest avec Données Réelles**

```bash
# Exécuter le backtest avec vraies données MT5
python demo_mt5_backtest.py
```

## 🛠️ Gestion d'Erreurs

L'adaptateur implémente une taxonomie d'erreurs sophistiquée :

### **Exceptions Python**

```python
from nautilus_trader.adapters.mt5.data import (
    Mt5DataError, Mt5ConnectionError, Mt5SubscriptionError, 
    Mt5DataRequestError, Mt5ParsingError
)

try:
    await data_client._subscribe_quote_ticks(instrument_id)
except Mt5ConnectionError as e:
    print(f"Erreur de connexion MT5: {e}")
except Mt5SubscriptionError as e:
    print(f"Erreur de subscription: {e}")
```

### **Logging Structuré**

```python
# Configuration du logging
import logging

logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - MT5 - %(levelname)s - %(message)s'
)

# Le data client utilisera automatiquement le logging configuré
```

## 📊 Métriques et Monitoring

L'adaptateur expose des métriques de performance :

- **Connexions** : HTTP et WebSocket avec retry automatique
- **Latence** : Temps de réponse des requêtes MT5
- **Subscriptions** : Nombre d'abonnements actifs par instrument
- **Throughput** : Messages WebSocket traités par seconde
- **Erreurs** : Classification et décompte par type

## 🔄 Reconnexion et Résilience

### **Gestion Automatique de Reconnection**

```python
# Configuration des paramètres de reconnection
config = Mt5DataClientConfig(
    connection_retry_attempts=3,
    connection_retry_delay=5,  # secondes
    reconnection_enabled=True,
    heartbeat_interval=30
)
```

### **Restore des Subscriptions**

- Les abonnements sont automatiquement restaurés après reconnexion
- État tracked : `pending` → `confirmed`
- Gestion des unsubscribe avec acks

## 📦 Dépendances

### **Core Dependencies**
- `nautilus-network`: Stack réseau optimisé
- `nautilus-core`: Types et modèles de base
- `tokio`: Runtime asynchrone
- `serde` / `serde_json`: Sérialisation/désérialisation
- `thiserror`: Gestion d'erreurs typée

### **WebSocket Dependencies**
- `tokio-tungstenite`: Client WebSocket asynchrone
- `futures-util`: Utilitaires pour async/await

### **Python Bindings**
- `pyo3`: Bindings Python pour Rust
- `pyo3_async_runtimes`: Runtime asynchrone pour PyO3

### **Development Dependencies**
- `axum`: Framework web pour tests
- `wiremock`: Mocking HTTP pour tests
- `maturin`: Build tool pour Python packages
- `tracing`: Logging structuré

## 🏆 État du Projet

### ✅ **Complété**
- Architecture Rust complète (HTTP/WS clients, parseurs, modèles)
- Clients Python (Instrument Provider, Data, Execution)
- Configurations enrichies avec tous les paramètres
- Gestion d'erreurs avec taxonomie complète
- Bindings PyO3 pour intégration Python
- Tests unitaires et d'intégration
- Documentation complète avec exemples

### 🔄 **En Production**
L'adaptateur MT5 est **production-ready** et peut être utilisé pour :

- Trading en temps réel sur MT5
- Backtests avec données historiques réelles
- Intégration complète avec Nautilus Trader
- Déploiement en production avec monitoring

## 📄 License

LGPL-3.0-or-later

## 🤝 Contribution

L'adaptateur est conçu pour être extensible :

- Nouveaux types d'instruments : Ajoutez dans `common/parse.rs`
- Nouveaux endpoints HTTP : Étendez `http/client.rs`
- Nouveaux types de messages WS : Ajoutez dans `websocket/messages.rs`
- Nouvelles configurations : Étendez dans `config/`

Pour contribuer, Consultez la `TODO_MT5_ADAPTER.md` pour les éléments restants.
