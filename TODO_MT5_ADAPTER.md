# TODO - MT5 Adapter (NautilusTrader)

Cette liste suit strictement la spécification "Adapters" (Rust core + couche Python + bindings PyO3 + tests).

## 1. Rust HTTP Client (core)

- [x] Structurer le client HTTP MT5 selon le pattern inner/outer:
  - [x] `Mt5HttpInnerClient` (état, HttpClient Nautilus, credentials, RetryManager, CancellationToken)
  - [x] `Mt5HttpClient` (wrapper `Arc<Mt5HttpInnerClient>`, clonable, sans logique métier)
- [x] Utiliser `nautilus_network::http::HttpClient` au lieu de `reqwest` direct.
- [ ] Implémenter l’authentification / signing MT5 si nécessaire.
- [ ] Implémenter les méthodes bas niveau `http_*`:
  - [ ] `http_get_symbols`
  - [ ] `http_get_symbol_info`
  - [ ] `http_get_account_info`
  - [ ] `http_get_positions`, `http_get_orders`, `http_get_history` (à détailler selon API cible)
- [ ] Implémenter les méthodes haut niveau:
  - [ ] `request_symbols`, `request_account`, `request_positions`, etc.
  - [ ] `submit_order`, `cancel_order`, `modify_order` (wrapper sur `http_*` + parse)
- [ ] Définir une taxonomie d’erreurs HTTP MT5 dédiée:
  - [ ] Enum d’erreurs avec variants retryable / non-retryable / fatal
  - [ ] Mapping des codes/réponses MT5 vers cet enum

## 2. Rust WebSocket Client

- [x] Implémenter un client WS MT5 dédié:
  - [x] Connexion / reconnexion
  - [x] Authentification (si nécessaire)
  - [x] Ping/Pong (frames + pings applicatifs)
- [x] Gestion des abonnements:
  - [x] États pending / confirmed
  - [x] Restore après reconnexion
  - [x] Unsubscribe correct (y compris acks)
- [x] Routing messages:
  - [x] Trades
  - [x] Quotes / order book
  - [x] Instrument status / events
- [ ] Définir erreurs WS MT5 dédiées:
  - [ ] Enum avec classification pour le retry

## 3. Modélisation & Parsing (Rust)

- [ ] `common/parse.rs`: parseurs communs (instruments, timestamps, prix, quantités)
- [ ] `http/models.rs` / `http/query.rs`:
  - [ ] Structs alignées avec le schéma MT5 (REST/bridge)
  - [ ] Query builders avec `derive_builder` + `serde` correct
- [ ] `http/parse.rs`:
  - [ ] Fonctions de conversion REST → modèles Nautilus
- [ ] `websocket/messages.rs` / `websocket/parse.rs`:
  - [ ] Types et parseurs pour les messages stream

## 4. Bindings PyO3 (Rust → Python)

- [x] Dans `crates/adapters/mt5/src/python/bindings.rs`:
  - [x] Exposer `Mt5HttpClient` et le(s) client(s) WS
  - [x] Marquer les structs nécessaires avec `#[pyclass]`
  - [x] Implémenter `#[pymethods]` avec `#[pyo3(name = "...")]`
  - [x] Utiliser `pyo3_async_runtimes::tokio::future_into_py` pour les méthodes async
- [ ] Dans `crates/adapters/mt5/src/python/mod.rs`:
  - [ ] `m.add_class::<...>()` pour tous les types exposés
  - [ ] Garder la surface synchronisée avec les besoins de la couche Python

## 5. Python - InstrumentProvider

- [ ] Compléter `Mt5InstrumentProviderConfig`:
  - [ ] Host / port / identifiants MT5 (ou bridge)
  - [ ] Options spécifiques (timeout, filtrage, etc.)
- [ ] Dans `Mt5InstrumentProvider`:
  - [ ] Utiliser le client PyO3 pour:
    - [ ] `load_all_async`: charger tous les symboles depuis MT5
    - [ ] `load_ids_async`: charger un sous-ensemble
    - [ ] `load_async`: charger un instrument ciblé
  - [ ] Remplacer le mapping simplifié par:
    - [ ] Détection FX / CFD / Futures / autres
    - [ ] Construction du bon type Nautilus (CurrencyPair, CFD, FuturesContract, ...)
- [ ] Gérer les erreurs MT5 → exceptions/cohérence Nautilus

## 6. Python - Data Client (Mt5DataClient)

- [ ] Brancher Mt5DataClient sur les bindings Rust:
  - [ ] `_connect` / `_disconnect` (si requis par LiveMarketDataClient base)
  - [ ] `_subscribe_*`:
    - [ ] trade ticks
    - [ ] quote ticks
    - [ ] order book (deltas + snapshots)
    - [ ] bars
    - [ ] instrument status / close
  - [ ] `_unsubscribe_*` correspondants
- [ ] `_request_*`:
  - [ ] instruments
  - [ ] quote ticks / trade ticks
  - [ ] bars
  - [ ] order book snapshot
  - [ ] Utiliser les endpoints HTTP exposés côté Rust
- [ ] Publier correctement sur le `MessageBus` les objets Nautilus construits à partir des réponses MT5

## 7. Python - Execution Client (Mt5ExecutionClient)

- [ ] Brancher Mt5ExecutionClient sur les bindings Rust:
  - [ ] `_submit_order` → endpoint MT5 (bridge) + parse réponse
  - [ ] `_modify_order`
  - [ ] `_cancel_order`
  - [ ] (optionnel) batch / cancel_all selon support MT5
- [ ] Implémenter:
  - [ ] `generate_order_status_report(s)`
  - [ ] `generate_fill_reports`
  - [ ] `generate_position_status_reports`
  - [ ] en se basant sur l’état retourné par MT5
- [ ] Gérer les erreurs/rejets de manière cohérente avec la taxonomie Rust

## 8. Python - Configs

- [ ] Enrichir:
  - [ ] `Mt5DataClientConfig`
  - [ ] `Mt5ExecClientConfig`
  - [ ] `Mt5InstrumentProviderConfig`
- [ ] Inclure:
  - [ ] Paramètres de connexion au bridge MT5
  - [ ] Identifiants / sécurité
  - [ ] Options de reconnection / timeouts

## 9. Erreurs & Logging

- [ ] Centraliser les erreurs MT5 côté Rust et les exposer côté Python
- [ ] S’assurer que:
  - [ ] Les erreurs HTTP/WS sont loggées clairement
  - [ ] Les erreurs Python reflètent la cause réelle (utile pour le debug)

## 10. Tests Rust

- [ ] HTTP:
  - [ ] Tests unitaires `http::parse`, `common::parse`
  - [ ] Tests intégration avec Axum mocks dans `crates/adapters/mt5/tests/http.rs`
  - [ ] Utiliser `test_data/http_*` complets
- [ ] WebSocket:
  - [ ] Tests unitaires `websocket::parse`
  - [ ] Tests intégration (auth, ping/pong, subscriptions, reconnexion, routing)
  - [ ] Utiliser `test_data/ws_*` si nécessaire

## 11. Tests Python

- [ ] Créer `tests/integration_tests/adapters/mt5/`:
  - [ ] Tester `Mt5InstrumentProvider` (avec clients Rust mockés)
  - [ ] Tester `Mt5DataClient` (subscriptions + request)
  - [ ] Tester `Mt5ExecutionClient` (submit/modify/cancel + rapports)
  - [ ] Tester `Mt5Factories` (wiring complet)
- [ ] S’assurer que:
  - [ ] Le comportement Python suit celui de la couche Rust (erreurs, reconnection, etc.)

## 12. Documentation

- [ ] Mettre à jour `crates/adapters/mt5/README.md`:
  - [ ] Architecture
  - [ ] Config / prérequis MT5
  - [ ] Exemples Rust + Python
- [ ] Ajouter un guide d’usage Python:
  - [ ] Exemple de création d’un client via `Mt5Factories`
  - [ ] Exemple de subscription data + envoi d’ordre
