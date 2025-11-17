# NT_MT5_Adapter

Adaptateur MetaTrader 5 pour NautilusTrader

## Structure de l'adaptateur

Cet adaptateur est divisé en deux parties principales :

1. **Partie Rust** : dans `crates/adapters/mt5/`
   - Gère les communications HTTP et WebSocket avec le pont MT5
   - Fournit des liaisons Python via PyO3
   - Implémente les fonctionnalités de bas niveau

2. **Partie Python** : dans `nautilus_trader/adapters/metatrader5/`
   - Intègre l'adaptateur dans le système NautilusTrader
   - Fournit les clients de données et d'exécution
   - Gère la configuration et les fournisseurs d'instruments

## Compilation

Pour compiler l'adaptateur avec les liaisons Python :

```bash
# Compiler en mode développement
maturin develop --features python-bindings

# Ou compiler en mode release
maturin build --release --features python-bindings
```

Ou avec Cargo directement :

```bash
cargo build --features python-bindings
```

## Configuration requise

- Rust (version spécifiée dans `rust-toolchain.toml`)
- Python 3.8+
- maturin ou pyo3 pour les liaisons Python
- Un service de pont MT5 en cours d'exécution (par exemple, un serveur HTTP/WS qui communique avec MT5)

## Tests

Pour tester l'adaptateur :

```bash
python Adapter_Backtest_Test.py
```

## Dépannage

Si vous rencontrez des problèmes :

1. Vérifiez que le pont MT5 est en cours d'exécution
2. Vérifiez que les liaisons Rust-Python sont correctement compilées
3. Vérifiez les paramètres de configuration (identifiants, URLs)

## Documentation

Référez-vous à `adapterdoc.txt` pour les spécifications détaillées de développement des adaptateurs.