# Configuration de l'adaptateur MT5 pour NautilusTrader
from nautilus_trader.adapters.mt5.config import (
    Mt5DataClientConfig,
    Mt5ExecutionClientConfig, 
    Mt5InstrumentProviderConfig
)
from nautilus_trader.adapters.mt5.common import Mt5Credential

# Configuration des credentials MT5
# IMPORTANT: Utilisez des credentials de test pour les backtests
credential = Mt5Credential(
    login="123456789",           # Votre login MT5 (entier)
    password="your_password",    # Votre mot de passe MT5
    server="MetaQuotes-Demo",    # Nom du serveur de trading
)

# Configuration du client de données
data_config = Mt5DataClientConfig(
    base_url="http://localhost:8080",  # URL de votre proxy MT5 bridge
    ws_url="ws://localhost:8080/ws",   # URL WebSocket (optionnel)
    http_timeout=30,                    # Timeout en secondes
    credential=credential,
)

# Configuration du client d'exécution
execution_config = Mt5ExecutionClientConfig(
    base_url="http://localhost:8080",
    http_timeout=30,
    simulate_orders=True,  # IMPORTANT: True pour backtest, False pour trading live
    risk_management_enabled=False,  # Désactiver pour backtest
    credential=credential,
)

# Configuration du fournisseur d'instruments
instrument_config = Mt5InstrumentProviderConfig(
    base_url="http://localhost:8080",
    http_timeout=30,
    # Filtres pour les instruments à inclure
    filter_currencies=["USD", "EUR", "GBP", "JPY", "CHF", "CAD"],  # Paires de devises principales
    filter_indices=["US30", "NAS100", "SPX500", "UK100", "GER40"], # Indices de trading
    filter_cfds=False,  # Désactiver les CFDs (plus complexe pour backtest)
    filter_futures=False,  # Désactiver les futures (optionnel)
    credential=credential,
)

# Configuration complète pour le backtest
backtest_config = {
    "data_client_configs": [data_config],
    "execution_client_configs": [execution_config], 
    "instrument_configs": [instrument_config],
}

print("Configuration MT5 chargée avec succès!")
print(f"- Login: {credential.login}")
print(f"- Serveur: {credential.server}")
print(f"- Mode simulation: {execution_config.simulate_orders}")