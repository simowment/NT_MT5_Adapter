#!/usr/bin/env python3
"""
MT5 Live Trading Test

Simple strategy that sends a market buy order every few seconds to test the adapter.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from decimal import Decimal

from nautilus_trader.common.enums import LogColor
from nautilus_trader.config import LiveDataEngineConfig
from nautilus_trader.config import LiveExecEngineConfig
from nautilus_trader.config import LoggingConfig
from nautilus_trader.config import StrategyConfig
from nautilus_trader.config import TradingNodeConfig
from nautilus_trader.live.node import TradingNode
from nautilus_trader.model.enums import OrderSide
from nautilus_trader.model.enums import TimeInForce
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import TraderId
from nautilus_trader.model.objects import Quantity
from nautilus_trader.trading.strategy import Strategy

from nautilus_mt5.config import (
    Mt5DataClientConfig,
    Mt5ExecClientConfig,
    Mt5InstrumentProviderConfig,
)
from nautilus_mt5.factories import Mt5LiveDataClientFactory, Mt5LiveExecClientFactory


BASE_URL = "http://localhost:5000"


class BuyEverySecondConfig(StrategyConfig, frozen=True):
    """Config for the test strategy."""

    instrument_id: str
    order_interval_secs: float = 5.0  # Order every 5 seconds
    trade_size: Decimal = Decimal("0.01")  # Minimum lot size
    max_orders: int = 3  # Stop after 3 orders


class BuyEverySecondStrategy(Strategy):
    """
    Simple test strategy that sends a market buy every N seconds.
    """

    def __init__(self, config: BuyEverySecondConfig) -> None:
        super().__init__(config)
        self.instrument_id = InstrumentId.from_str(config.instrument_id)
        self.order_interval_secs = config.order_interval_secs
        self.trade_size = config.trade_size
        self.max_orders = config.max_orders
        self.orders_sent = 0

    def on_start(self) -> None:
        self.log.info(
            "Strategy starting - will send buy orders periodically",
            color=LogColor.GREEN,
        )

        # Request instrument to be loaded
        self.request_instrument(self.instrument_id)

        # Subscribe to quote ticks to get price updates
        self.subscribe_quote_ticks(self.instrument_id)

        # Schedule first order with delay to allow instrument loading
        from datetime import timedelta

        self.clock.set_timer(
            name="order_timer",
            interval=timedelta(seconds=self.order_interval_secs),
            callback=self._on_timer,
        )

    def on_quote_tick(self, tick) -> None:
        self.log.info(
            f"Quote: {tick.instrument_id} bid={tick.bid_price} ask={tick.ask_price}"
        )

    def _on_timer(self, event) -> None:
        if self.orders_sent >= self.max_orders:
            self.log.warning(f"Max orders ({self.max_orders}) reached, stopping timer")
            self.clock.cancel_timer("order_timer")
            return

        self.log.info(f"Timer fired - sending buy order #{self.orders_sent + 1}")
        self._send_buy_order()

    def _send_buy_order(self) -> None:
        instrument = self.cache.instrument(self.instrument_id)
        if not instrument:
            self.log.error(f"Instrument {self.instrument_id} not in cache")
            return

        order = self.order_factory.market(
            instrument_id=self.instrument_id,
            order_side=OrderSide.BUY,
            quantity=Quantity(float(self.trade_size), instrument.size_precision),
            time_in_force=TimeInForce.IOC,  # Immediate or cancel
        )

        self.submit_order(order)
        self.orders_sent += 1
        self.log.info(f"Submitted order: {order}", color=LogColor.BLUE)

    def on_order_accepted(self, event) -> None:
        self.log.info(f"Order ACCEPTED: {event}", color=LogColor.GREEN)

    def on_order_rejected(self, event) -> None:
        self.log.error(f"Order REJECTED: {event}")

    def on_order_filled(self, event) -> None:
        self.log.info(f"Order FILLED: {event}", color=LogColor.GREEN)

    def on_stop(self) -> None:
        self.log.info(f"Strategy stopped. Total orders sent: {self.orders_sent}")


def main():
    # Instrument provider config
    provider_config = Mt5InstrumentProviderConfig(
        base_url=BASE_URL,
    )

    # Data client config
    data_config = Mt5DataClientConfig(
        base_url=BASE_URL,
        instrument_provider=provider_config,
        poll_interval_ms=500,  # 500ms polling
    )

    # Execution client config
    exec_config = Mt5ExecClientConfig(
        base_url=BASE_URL,
        instrument_provider=provider_config,
    )

    # Trading node config
    node_config = TradingNodeConfig(
        trader_id=TraderId("TESTER-001"),
        logging=LoggingConfig(log_level="INFO"),
        data_engine=LiveDataEngineConfig(
            time_bars_timestamp_on_close=True,
        ),
        exec_engine=LiveExecEngineConfig(),
        data_clients={"MT5": data_config},
        exec_clients={"MT5": exec_config},
        timeout_connection=30.0,
        timeout_reconciliation=10.0,
        timeout_portfolio=10.0,
        timeout_disconnection=10.0,
    )

    # Build node
    node = TradingNode(config=node_config)

    # Register factories
    node.add_data_client_factory("MT5", Mt5LiveDataClientFactory)
    node.add_exec_client_factory("MT5", Mt5LiveExecClientFactory)

    # Strategy config
    strategy_config = BuyEverySecondConfig(
        instrument_id="EURUSD.MT5",
        order_interval_secs=5.0,
        trade_size=Decimal("0.01"),
        max_orders=3,
    )
    strategy = BuyEverySecondStrategy(config=strategy_config)
    node.trader.add_strategy(strategy)

    # Build and run
    node.build()

    try:
        node.run()
    except KeyboardInterrupt:
        print("\nShutting down...")
    finally:
        node.dispose()


if __name__ == "__main__":
    main()
