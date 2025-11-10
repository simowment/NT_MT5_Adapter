# -*- coding: utf-8 -*-
"""
Démonstration de l'utilisation de l'adaptateur MT5 avec des données réelles pour les backtests.

Ce script montre comment charger des données historiques réelles de MetaTrader 5
et les utiliser dans un backtest avec Nautilus Trader.
"""
import asyncio
from decimal import Decimal
from datetime import datetime
from typing import List
import pandas as pd

# Imports de Nautilus Trader
from nautilus_trader.backtest.engine import BacktestEngine, BacktestEngineConfig
from nautilus_trader.model.data import Bar, QuoteTick, TradeTick
from nautilus_trader.model.identifiers import InstrumentId, Venue
from nautilus_trader.model.objects import Price, Quantity, Money
from nautilus_trader.model.enums import (
    AccountType,
    OMSType,
    TradeSide
)
from nautilus_trader.core.datetime import millis_to_nanos

# Imports de l'adaptateur MT5

class Mt5BacktestDataLoader:
    """
    Chargeur de données pour les backtests avec des données historiques MT5 réelles.
    """
    
    def __init__(self, mt5_client):
        self.mt5_client = mt5_client
        
    def load_historical_bars(self, instrument_id: InstrumentId, start: datetime, end: datetime, timeframe: str = "M1") -> List[Bar]:
        """
        Charge des barres historiques depuis MT5 pour le backtest.
        
        :param instrument_id: L'identifiant de l'instrument
        :param start: Date de début
        :param end: Date de fin  
        :param timeframe: Timeframe (M1, M5, H1, etc.)
        :return: Liste de barres
        """
        # Dans un scénario réel, cette méthode appellerait l'API MT5 pour récupérer
        # les données historiques et les convertirait en objets Bar Nautilus
        print(f"Chargement des données historiques pour {instrument_id} du {start} au {end} (timeframe: {timeframe})")
        
        # Simulation de données pour la démonstration
        # Dans la réalité, vous utiliseriez l'API MT5 pour récupérer les données réelles
        bars = []
        current_time = start
        
        # Génération de données de démonstration
        price = 1.1000
        for i in range(100):  # 100 barres pour la démo
            open_price = price
            high_price = price + (0.005 * (i % 3))
            low_price = price - (0.005 * (i % 2))
            close_price = price + (0.0001 * ((i + 1) % 2))
            
            bar = Bar(
                bar_type=Bar.bar_type(instrument_id, timeframe, "EXTERNAL"),
                open=Price(open_price, precision=5),
                high=Price(high_price, precision=5),
                low=Price(low_price, precision=5),
                close=Price(close_price, precision=5),
                volume=Quantity(1_000, precision=0),
                ts_event=millis_to_nanos(int(current_time.timestamp() * 1000)),
                ts_init=millis_to_nanos(int(current_time.timestamp() * 1000)),
            )
            
            bars.append(bar)
            current_time += pd.Timedelta(minutes=1)
            price = close_price # Prix de clôture devient le prix de départ suivant
            
        return bars
    
    def load_historical_quotes(self, instrument_id: InstrumentId, start: datetime, end: datetime) -> List[QuoteTick]:
        """
        Charge les ticks de cotation historiques depuis MT5.
        """
        print(f"Chargement des quotes historiques pour {instrument_id}")
        
        # Simulation de données de démonstration
        quotes = []
        current_time = start
        
        for i in range(50):  # 50 quotes pour la démo
            bid_price = 1.1000 + (0.0001 * (i % 10))
            ask_price = bid_price + 0.0001  # Spread de 1 pip
            
            quote = QuoteTick(
                instrument_id=instrument_id,
                bid=Price(bid_price, precision=5),
                ask=Price(ask_price, precision=5),
                bid_size=Quantity(1_000_000, precision=0),
                ask_size=Quantity(1_000_000, precision=0),
                ts_event=millis_to_nanos(int(current_time.timestamp() * 1000)),
                ts_init=millis_to_nanos(int(current_time.timestamp() * 1000)),
            )
            
            quotes.append(quote)
            current_time += pd.Timedelta(seconds=10)
            
        return quotes
    
    def load_historical_trades(self, instrument_id: InstrumentId, start: datetime, end: datetime) -> List[TradeTick]:
        """
        Charge les ticks de trade historiques depuis MT5.
        """
        print(f"Chargement des trades historiques pour {instrument_id}")
        
        # Simulation de données de démonstration
        trades = []
        current_time = start
        
        for i in range(30):  # 30 trades pour la démo
            trade = TradeTick(
                instrument_id=instrument_id,
                price=Price(1.1000 + (0.0001 * (i % 20)), precision=5),
                size=Quantity(100_000, precision=0),
                aggressor_side=TradeSide.BUY if i % 2 == 0 else TradeSide.SELL,
                trade_id=f"trade_{i}",
                ts_event=millis_to_nanos(int(current_time.timestamp() * 100)),
                ts_init=millis_to_nanos(int(current_time.timestamp() * 100)),
            )
            
            trades.append(trade)
            current_time += pd.Timedelta(seconds=15)
            
        return trades

async def run_real_mt5_backtest():
    """
    Exécute un backtest avec des données historiques MT5 réelles.
    """
    print("Initialisation du backtest avec données MT5 réelles...")
    
    # Configuration du moteur de backtest
    config = BacktestEngineConfig(
        log_level="INFO",
        loop_debug=False,
    )
    
    engine = BacktestEngine(config=config)
    
    try:
        # Création d'un instrument de démonstration (dans un vrai scénario, ce serait un instrument MT5 réel)
        from nautilus_trader.test_kit.providers import TestInstrumentProvider
        instrument = TestInstrumentProvider.default_fx_ccy("EUR/USD")
        
        # Création du loader de données MT5
        # Note: Dans la réalité, vous auriez un client MT5 pour accéder aux données historiques
        # Pour cette démo, nous simulons le processus
        mt5_data_loader = Mt5BacktestDataLoader(mt5_client=None)
        
        # Chargement des données historiques MT5
        start_date = datetime(2022, 1, 1)
        end_date = datetime(2022, 1, 2)
        
        print("Chargement des données historiques MT5...")
        bars = mt5_data_loader.load_historical_bars(instrument.id, start_date, end_date, "M1")
        quotes = mt5_data_loader.load_historical_quotes(instrument.id, start_date, end_date)
        trades = mt5_data_loader.load_historical_trades(instrument.id, start_date, end_date)
        
        # Ajout de la venue MT5
        mt5_venue = Venue("MT5")
        engine.add_venue(
            venue=mt5_venue,
            oms_type=OMSType.HEDGING,
            account_type=AccountType.MARGIN,
            base_currency=instrument.base_currency,
            starting_balances=[Money(100_000, instrument.base_currency)],
        )
        
        # Ajout de l'instrument
        engine.add_instrument(instrument)
        
        # Ajout des données de backtest (bars, quotes, trades)
        print("Ajout des données de backtest...")
        for bar in bars:
            engine.add_data([bar], mt5_venue)
        
        for quote in quotes:
            engine.add_data([quote], mt5_venue)
        
        for trade in trades:
            engine.add_data([trade], mt5_venue)
        
        # Création d'une stratégie simple pour le backtest
        from nautilus_trader.test_kit.strategies import EMACross
        strategy = EMACross(
            instrument_id=instrument.id,
            bar_type=Bar.bar_type(instrument.id, "1-MINUTE", "EXTERNAL"),
            trade_size=Decimal("1000"),
            fast_ema_period=10,
            slow_ema_period=20,
        )
        
        engine.add_strategy(strategy)
        
        print("Exécution du backtest...")
        engine.run()
        
        print("Backtest terminé avec succès!")
        print(f"Résultats: {engine.iteration} itérations")
        
        # Affichage des résultats du portefeuille
        portfolio = engine.portfolio
        print(f"Valeur du portefeuille finale: {portfolio.unrealized_pnl(instrument.id)}")
        
    finally:
        await engine.dispose()



if __name__ == "__main__":
    print("Démonstration de l'utilisation de l'adaptateur MT5 avec données réelles pour backtest")
    print("=" * 80)
    
    print("\n" + "=" * 80)
    print("Exécution du backtest de démonstration avec données MT5 simulées...")
    
    # Exécution du backtest
    asyncio.run(run_real_mt5_backtest())
    
    print("\nDémonstration terminée avec succès!")
    print("\nPOUR UTILISER AVEC DES DONNÉES MT5 RÉELLES:")
    print("1. Assurez-vous que le serveur MT5 Bridge est en cours d'exécution")
    print("2. Compilez l'adaptateur MT5: cargo build -p nautilus-adapters-mt5")
    print("3. Configurez les identifiants MT5 dans la configuration")
    print("4. Utilisez les méthodes de chargement de données pour récupérer les données historiques MT5")

