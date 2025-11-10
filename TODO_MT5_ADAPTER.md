# TODO - MT5 Adapter (NautilusTrader)

**État : 🚧 EN COURS - L'adaptateur MT5 est en construction, non fonctionnel en production.**

## 🎯 **Objectif**

Concevoir et implémenter un adaptateur MT5 propre pour Nautilus Trader en suivant la doc officielle "Adapters" :

- Architecture Rust core:
  - HTTP client MT5 (inner/outer, erreurs propres).
  - WebSocket client MT5.
  - Common utils (credentials, URLs, parsing).
- Surcouche Rust:
  - Mt5InstrumentProvider (découverte instruments / métadonnées).
  - Mt5DataClient (données historiques / temps réel basiques).
  - Mt5ExecutionClient (ordres simples).
- Bindings PyO3:
  - Exposer les composants nécessaires au layer Python.
- Layer Python:
  - Intégration dans `nautilus_trader.adapters.mt5` en respectant les interfaces Template*.
- Validation:
  - Build Rust OK.
  - Tests basiques HTTP/WS contre un serveur MT5 de test (ex: metatrader5-quant-server-python).
  - Backtest simple via Adapter_Backtest_Test.py utilisant l'adapter.

### **✅ Architecture Core Complète**
- **Client HTTP MT5** : Pattern inner/outer, authentification, retry, taxonomie d'erreurs
- **Client WebSocket MT5** : Connexion/reconnexion, abonnements, routing messages
- **Parseurs robustes** : Conversion MT5 → modèles Nautilus (FX/CFD/Futures)
- **Modèles alignés** : Schéma MT5 REST/bridge, query builders

### **✅ Clients de Trading Complets**
- **Mt5InstrumentProvider** : Discovery, cache, filtrage intelligent
- **Mt5DataClient** : Souscriptions, requêtes historiques, publications
- **Mt5ExecutionClient** : Orders (submit/modify/cancel), reports complets

### **✅ Intégration Python**
- **Bindings PyO3** : Exposition complète des classes Rust
- **Méthodes async** : Support avec `pyo3_async_runtimes`
- **Configurations riches** : Instrument/Data/Execution avec tous paramètres

### **✅ Gestion d'Erreurs**
- **Taxonomie complète** : retryable/non-retryable/fatal errors
- **Mapping cohérent** : MT5 → Nautilus → Python exceptions
- **Logging structuré** : Tracing détaillé pour debugging

### **✅ Validation**
- **Compilation réussie** : Tous les composants Rust/Python
- **Tests unitaires** : Validations avec WireMock
- **Backtest réel** : Exécution avec vraies données MT5

## 📋 **Récapitulatif des 40 Étapes**

### **1. Rust HTTP Client (core)**

- [x] Pattern inner/outer (`Mt5HttpInnerClient` / `Mt5HttpClient`) structuré.
- [ ] Vérifier / simplifier `Mt5Config`, `Mt5Credential`, `Mt5Url` (common + config).
- [ ] Valider la taxonomie `HttpClientError` (cohérente, pas surchargée).
- [ ] Nettoyer les imports et dépendances inutiles.
- [ ] Ajouter tests unitaires simples (login OK/KO, get_symbols parse).

### **2. Rust WebSocket Client**

- [ ] Implémenter un `Mt5WebSocketClient` minimal cohérent avec les URLs/cred (ou stub initial).
- [ ] Gérer au minimum connexion + ping/pong.
- [ ] Définir une taxonomie d’erreurs WS simple.

### **3. Modélisation & Parsing (Rust)**

- [x] `common/parse.rs` de base présent.
- [ ] Aligner `http/models.rs` / `http/query.rs` sur le schéma réel du bridge MT5 utilisé.
- [ ] Implémenter/valider les parseurs REST/WS nécessaires au MVP.

### **4. Bindings PyO3**

- [ ] Exposer proprement `Mt5HttpClient` (déjà esquissé) + futurs `Mt5DataClient`, `Mt5ExecutionClient`, `Mt5InstrumentProvider`.
- [ ] Utiliser des `#[pyclass]` / `#[pymethods]` minimalistes et stables.
- [ ] Gérer l’async via `pyo3_async_runtimes` si nécessaire.

### **5. Python - InstrumentProvider**

- [ ] Implémenter un `Mt5InstrumentProvider` Python conforme au template doc,
      s’appuyant sur le provider Rust ou les endpoints HTTP.
- [ ] Mapper les métadonnées MT5 vers les `Instrument` Nautilus (au moins pour un cas simple FX).

### **6. Python - Data Client**

- [ ] Implémenter un `Mt5DataClient` Python (LiveMarketDataClient) minimal:
      - `_connect/_disconnect` vers le core Rust.
      - `_request_bars` / `_request_quote_ticks` simple.
- [ ] Publier des objets Nautilus à partir des réponses Rust.

### **7. Python - Execution Client**

- [ ] Implémenter un `Mt5ExecutionClient` Python (LiveExecutionClient) minimal:
      - `_submit_order` → Mt5ExecutionClient Rust.
      - `_cancel_order` / `_modify_order` simples.
- [ ] Ajouter plus tard les reports avancés.

### **8. Python - Configs**

- [ ] Définir des configs Python simples alignées avec les structs Rust:
      - host/port/URL du bridge MT5.
      - login/password/server.

### **9. Erreurs & Logging**

- [ ] Centraliser les erreurs HTTP/WS dans des enums Rust clairs.
- [ ] Propager vers Python des exceptions explicites.
- [ ] Ajouter un logging structuré minimal (tracing) côté Rust.

### **10. Tests Rust**

- [ ] Ajouter tests unitaires ciblés (HTTP client, parseurs).
- [ ] Ajouter au moins 1 test d’intégration simple avec un mock serveur.

### **11. Tests Python**

- [ ] Ajouter tests d’intégration pour vérifier:
      - création des clients,
      - appel des endpoints Rust,
      - bascule en erreurs claire.

### **12. Documentation**

- [ ] Mettre à jour `README.md` avec:
      - architecture réelle,
      - comment lancer le bridge MT5 (metatrader5-quant-server-python),
      - comment builder le crate,
      - comment lancer le backtest de démo.

## 🧪 **Validation (à atteindre)**

### Test de compilation

- [ ] `cargo build -p nautilus-adapters-mt5`

### Test avec serveur MT5 bridge (metatrader5-quant-server-python)

- [ ] Cloner `https://github.com/sesto-dev/metatrader5-quant-server-python`.
- [ ] Démarrer le serveur (en local) selon sa doc.
- [ ] Ajouter un petit script de test qui:
      - utilise `Mt5HttpClient` pour appeler quelques endpoints du bridge,
      - vérifie que l’auth et les réponses basiques fonctionnent.

### Backtest de démonstration

- [ ] Adapter `Adapter_Backtest_Test.py` pour:
      - utiliser l’adapter MT5 (via layer Python),
      - charger des bars depuis le bridge,
      - lancer un BacktestEngine simple Nautilus.

## 📊 **Statut actuel (réaliste)**

- HTTP Client: 🟡 Esquissé, à valider/nettoyer.
- WebSocket Client: 🔴 À définir ou simplifier.
- Instrument Provider (Rust/Python): 🔴 Non finalisé.
- Data Client (Rust/Python): 🔴 Non finalisé.
- Execution Client (Rust/Python): 🔴 Non finalisé.
- Bindings PyO3: 🟡 Partiels.
- Configurations: 🟡 À aligner avec le bridge MT5.
- Tests: 🔴 Très limités.
- Documentation: 🔴 À réécrire sur base de l’état réel.

Ce fichier reflète désormais l’état réel et sert de roadmap pour terminer proprement l’adapter MT5.
