#!/usr/bin/env python3
"""
VRAI backtest NautilusTrader + MT5

Prérequis:
- pip install "nautilus-trader[all]"
- MT5_Rest_Server.py lancé sur http://localhost:5000
"""

import asyncio
from datetime import datetime

import requests

# On utilise le vrai NautilusTrader (installé via pip)
from nautilus_trader.backtest.config import BacktestEngineConfig
from nautilus_trader.backtest.engine import BacktestEngine
from nautilus_trader.config import LoggingConfig
from nautilus_trader.model.enums import OrderSide, OrderType
from nautilus_trader.model.objects import Price, Quantity
from nautilus_trader.model.identifiers import InstrumentId, StrategyId
from nautilus_trader.trading.strategy import Strategy


BASE_URL = "http://localhost:5000"


class Mt5RestProxy:
    """
    Client minimal vers ton serveur REST MT5.
    Utilisé comme backend pour alimenter le backtest.
    """

    def __init__(self, base_url: str):
        self.base_url = base_url.rstrip("/")
        self.session = requests.Session()

    def _post(self, name: str, payload: dict):
        url = f"{self.base_url}/api/{name}"
        resp = self.session.post(url, json=payload, timeout=10)
        resp.raise_for_status()
        return resp.json()

    def login(self, login: str, password: str, server: str):
        return self._post("login", {
            "login": login,
            "password": password,
            "server": server,
        })

    def copy_rates_from(self, symbol: str, timeframe: str, start: int, count: int):
        return self._post("copy_rates_from", {
            "symbol": symbol,
            "timeframe": timeframe,
            "start": start,
            "count": count,
        })


class Mt5DemoStrategy(Strategy):
    """
    Stratégie ultra simple:
    - Ouvre un BUY au premier bar
    - Ferme sur stop/tp simplifié
    """

    def __init__(self, instrument_id: InstrumentId):
        self.instrument_id = instrument_id
        self.position_opened = False

    @classmethod
    def create(cls, config, dependencies):
        # Config & wiring via Nautilus (non utilisé à fond ici)
        instrument_id = InstrumentId.from_str("EURUSD.MT5")
        return cls(instrument_id)

    @property
    def id(self) -> StrategyId:
        return StrategyId("MT5-DEMO")

    async def on_start(self):
        self.log.info("MT5 demo strategy started")

    async def on_stop(self):
        self.log.info("MT5 demo strategy stopped")

    async def on_bar(self, bar):
        if bar.instrument_id != self.instrument_id:
            return

        if not self.position_opened:
            # BUY 0.1 lots
            order = await self.send_order(
                instrument_id=self.instrument_id,
                side=OrderSide.BUY,
                order_type=OrderType.MARKET,
                quantity=Quantity.from_int(10000),
            )
            self.log.info(f"Submitted BUY order: {order}")
            self.position_opened = True


async def run_backtest():
    # 1. Vérifier accès serveur MT5 REST
    try:
        r = requests.post(f"{BASE_URL}/api/login", json={
            "login": "demo", "password": "demo", "server": "demo.mt5.com",
        }, timeout=5)
        print(f"MT5 REST login status: {r.status_code}")
    except Exception as e:
        print(f"Erreur connexion MT5 REST: {e}")
        return

    # 2. Config backtest Nautilus
    engine_config = BacktestEngineConfig(
        start_time=datetime(2024, 1, 1),
        end_time=datetime(2024, 1, 2),
        logging=LoggingConfig(),
    )

    engine = BacktestEngine(engine_config)

    # 3. Enregistrer un instrument factice MT5 (normalement via provider)
    instrument_id = InstrumentId.from_str("EURUSD.MT5")
    # Dans un vrai setup on chargerait l'instrument depuis Mt5InstrumentProvider
    # Ici on suppose qu'il est déjà connu dans la data feed Nautilus

    # 4. Ajouter la stratégie
    engine.add_strategy(Mt5DemoStrategy, config={})

    # 5. Charger des données depuis ton proxy MT5 et les injecter
    proxy = Mt5RestProxy(BASE_URL)
    # Exemple: timestamp fixe; à adapter selon ton serveur
    start_timestamp = 1704067200  # 2024-01-01 00:00:00
    bars = proxy.copy_rates_from("EURUSD", "M1", start_timestamp, 500)

    # Transformer les bars REST en events Nautilus
    # NOTE: Il faut mapper correctement vers Bar dataclass Nautilus.
    # Ici on fait un pseudo-mapping minimal; à ajuster selon le schéma exact.
    from nautilus_trader.model.data import Bar, BarType

    bar_type = BarType.from_str("EURUSD.MT5-1m-BID-EXTERNAL")
    events = []
    for b in bars:
        # Attendu: b = {time, open, high, low, close, tick_volume}
        ts = int(b.get("time", start_timestamp)) * (10**9)
        events.append(Bar(
            bar_type=bar_type,
            open=Price.from_float(float(b["open"])),
            high=Price.from_float(float(b["high"])),
            low=Price.from_float(float(b["low"])),
            close=Price.from_float(float(b["close"])),
            volume=Quantity.from_int(int(b.get("tick_volume", 1))),
            ts_event=ts,
            ts_init=ts,
        ))

    # Injecter les events dans l’engine
    for e in events:
        engine.process_data(e)

    # 6. Run
    engine.run()

    # 7. Résultats
    report = engine.generate_report()
    print(report.json(indent=2))


def main():
    asyncio.run(run_backtest())


if __name__ == "__main__":
    main()