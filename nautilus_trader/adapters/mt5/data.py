# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------

"""
The data client for the MetaTrader 5 integration.

"""


from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import MessageBus
from nautilus_trader.core.correctness import PyCondition
from nautilus_trader.live.data_client import LiveMarketDataClient
from nautilus_trader.model.data import BarType
from nautilus_trader.model.data import DataType
from nautilus_trader.model.data import InstrumentStatusUpdate
from nautilus_trader.model.data import QuoteTick
from nautilus_trader.model.data import TradeTick
from nautilus_trader.model.identifiers import ClientId
from nautilus_trader.model.identifiers import InstrumentId

# Import pour gestion d'erreurs MT5
import traceback
import logging
from typing import Optional, Union


class Mt5DataError(Exception):
    """Base exception for MT5 data client errors."""
    pass


class Mt5ConnectionError(Mt5DataError):
    """Raised when connection to MT5 fails."""
    pass


class Mt5SubscriptionError(Mt5DataError):
    """Raised when subscription to data fails."""
    pass


class Mt5DataRequestError(Mt5DataError):
    """Raised when data request fails."""
    pass


class Mt5ParsingError(Mt5DataError):
    """Raised when parsing MT5 data fails."""
    pass


class Mt5DataClient(LiveMarketDataClient):
    """
    MetaTrader 5 market data client (skeleton).

    This class is wired to fit the NautilusTrader LiveMarketDataClient interface
    and should delegate to the Rust MT5 HTTP/WS clients exposed via PyO3.
    """

    def __init__(self, loop, http_client, ws_client, msgbus, cache, clock):
        super().__init__(
            client_id=ClientId("MT5"),
            venue=http_client.venue if hasattr(http_client, 'venue') else "MT5",
            loop=loop,
            clock=clock,
            msgbus=msgbus,
            cache=cache,
        )

        PyCondition.not_none(http_client, "http_client")
        PyCondition.not_none(ws_client, "ws_client")

        self._http_client = http_client
        self._ws_client = ws_client
        self._connected = False
        
        # Configuration du logging MT5
        self._setup_mt5_logging()

    def _setup_mt5_logging(self):
        """Configure le logging pour l'adaptateur MT5."""
        # Configurer le niveau de log basé sur la configuration
        if hasattr(self, '_config') and hasattr(self._config, 'enable_logging'):
            log_level = logging.DEBUG if self._config.enable_logging else logging.INFO
        else:
            log_level = logging.INFO
            
        # Ajouter un handler spécialisé pour MT5
        mt5_logger = logging.getLogger(f"nautilus_trader.adapters.mt5.data")
        if not mt5_logger.handlers:
            handler = logging.StreamHandler()
            formatter = logging.Formatter(
                '%(asctime)s - %(name)s - MT5 - %(levelname)s - %(message)s'
            )
            handler.setFormatter(formatter)
            mt5_logger.addHandler(handler)
            mt5_logger.setLevel(log_level)

    def _log_connection_error(self, error: Exception, operation: str):
        """Log les erreurs de connexion avec contexte détaillé."""
        self._log.error(f"MT5 Connection Error during {operation}: {error}")
        self._log.error(f"Error type: {type(error).__name__}")
        self._log.error(f"Error details: {str(error)}")
        
        # Log le stack trace complet pour le debugging
        if hasattr(self, '_config') and self._config.enable_logging:
            self._log.debug(f"Stack trace: {traceback.format_exc()}")

    def _log_subscription_error(self, error: Exception, instrument_id: InstrumentId, subscription_type: str):
        """Log les erreurs de subscription avec contexte."""
        self._log.error(f"MT5 Subscription Error for {instrument_id} ({subscription_type}): {error}")
        self._log.error(f"Subscription details: {subscription_type}")
        
        # Log les tentatives de retry si configuré
        if hasattr(self, '_config') and hasattr(self._config, 'connection_retry_attempts'):
            self._log.info(f"Will retry up to {self._config.connection_retry_attempts} times")

    def _handle_mt5_error(self, error: Exception, context: str, raise_error: bool = True) -> Optional[Exception]:
        """Gestion centralisée des erreurs MT5."""
        error_type = type(error).__name__
        
        # Log l'erreur avec contexte
        self._log.error(f"MT5 Error in {context}: {error}")
        self._log.error(f"Error type: {error_type}")
        
        # Mapping des erreurs vers des exceptions MT5 appropriées
        if "connection" in str(error).lower() or "timeout" in str(error).lower():
            wrapped_error = Mt5ConnectionError(f"MT5 connection failed during {context}: {error}")
        elif "subscription" in str(error).lower() or "subscribe" in str(error).lower():
            wrapped_error = Mt5SubscriptionError(f"MT5 subscription failed during {context}: {error}")
        elif "parse" in str(error).lower() or "decode" in str(error).lower():
            wrapped_error = Mt5ParsingError(f"MT5 data parsing failed during {context}: {error}")
        elif "request" in str(error).lower() or "data" in str(error).lower():
            wrapped_error = Mt5DataRequestError(f"MT5 data request failed during {context}: {error}")
        else:
            wrapped_error = Mt5DataError(f"MT5 error during {context}: {error}")
        
        # Log les détails supplémentaires
        if hasattr(self, '_config') and self._config.enable_logging:
            self._log.debug(f"Original error details: {str(error)}")
            self._log.debug(f"Stack trace: {traceback.format_exc()}")
        
        if raise_error:
            raise wrapped_error
        else:
            return wrapped_error

    def connect(self):
        self._log.info("Connecting to MT5...")
        try:
            # Authentification HTTP
            import asyncio
            loop = asyncio.get_event_loop()
            loop.run_until_complete(self._http_client.login())
            
            # Connexion WebSocket
            loop.run_until_complete(self._ws_client.connect())
            loop.run_until_complete(self._ws_client.authenticate())
            
            self._connected = True
            self._log.info("Connected to MT5 successfully.")
        except Exception as e:
            self._log.error(f"Failed to connect to MT5: {e}")
            raise

    def disconnect(self):
        self._log.info("Disconnecting from MT5...")
        if self._connected:
            try:
                import asyncio
                loop = asyncio.get_event_loop()
                if self._ws_client:
                    loop.run_until_complete(self._ws_client.disconnect())
                self._connected = False
                self._log.info("Disconnected from MT5.")
            except Exception as e:
                self._log.error(f"Error during disconnection: {e}")
        else:
            self._log.info("Already disconnected.")

    async def _subscribe_trade_ticks(self, instrument_id: InstrumentId):
        self._log.info(f"Subscribing to trade ticks for {instrument_id}")
        try:
            if self._connected and self._ws_client:
                symbol = str(instrument_id)
                await self._ws_client.subscribe_trades(symbol)
                self._log.debug(f"Trade ticks subscription confirmed for {symbol}")
            else:
                self._log.warning("Not connected to MT5 WebSocket")
        except Exception as e:
            self._log.error(f"Failed to subscribe to trade ticks for {instrument_id}: {e}")
            raise

    async def _subscribe_quote_ticks(self, instrument_id: InstrumentId):
        self._log.info(f"Subscribing to quote ticks for {instrument_id}")
        try:
            if self._connected and self._ws_client:
                symbol = str(instrument_id)
                await self._ws_client.subscribe_quotes(symbol)
                self._log.debug(f"Quote ticks subscription confirmed for {symbol}")
            else:
                self._log.warning("Not connected to MT5 WebSocket")
        except Exception as e:
            self._log.error(f"Failed to subscribe to quote ticks for {instrument_id}: {e}")
            raise

    async def _subscribe_bars(self, bar_type: BarType):
        self._log.info(f"Subscribing to bars for {bar_type}")
        try:
            if self._connected and self._ws_client:
                symbol = str(bar_type.instrument_id)
                # For bars, we might use a different subscription pattern
                await self._ws_client.subscribe_quotes(symbol)  # Use quotes as proxy for now
                self._log.debug(f"Bars subscription confirmed for {bar_type}")
            else:
                self._log.warning("Not connected to MT5 WebSocket")
        except Exception as e:
            self._log.error(f"Failed to subscribe to bars for {bar_type}: {e}")
            raise

    async def _unsubscribe_trade_ticks(self, instrument_id: InstrumentId):
        self._log.info(f"Unsubscribing from trade ticks for {instrument_id}")
        try:
            if self._connected and self._ws_client:
                symbol = str(instrument_id)
                await self._ws_client.unsubscribe(f"trade:{symbol}")
                self._log.debug(f"Trade ticks unsubscribed for {symbol}")
            else:
                self._log.warning("Not connected to MT5 WebSocket")
        except Exception as e:
            self._log.error(f"Failed to unsubscribe from trade ticks for {instrument_id}: {e}")
            raise

    async def _unsubscribe_quote_ticks(self, instrument_id: InstrumentId):
        self._log.info(f"Unsubscribing from quote ticks for {instrument_id}")
        try:
            if self._connected and self._ws_client:
                symbol = str(instrument_id)
                await self._ws_client.unsubscribe(f"quote:{symbol}")
                self._log.debug(f"Quote ticks unsubscribed for {symbol}")
            else:
                self._log.warning("Not connected to MT5 WebSocket")
        except Exception as e:
            self._log.error(f"Failed to unsubscribe from quote ticks for {instrument_id}: {e}")
            raise

    async def _unsubscribe_bars(self, bar_type: BarType):
        self._log.info(f"Unsubscribing from bars for {bar_type}")
        try:
            if self._connected and self._ws_client:
                symbol = str(bar_type.instrument_id)
                await self._ws_client.unsubscribe(f"quote:{symbol}")
                self._log.debug(f"Bars unsubscribed for {bar_type}")
            else:
                self._log.warning("Not connected to MT5 WebSocket")
        except Exception as e:
            self._log.error(f"Failed to unsubscribe from bars for {bar_type}: {e}")
            raise

    async def _request_data(self, data_type: DataType, correlation_id: str, start: object = None, end: object = None):
        self._log.info(f"Requesting data: {data_type}")
        try:
            if not self._connected:
                self._log.warning("Not connected to MT5, cannot request data")
                return
                
            # Route to specific request method based on data type
            if data_type.type_name == "quote_ticks":
                await self._request_quote_ticks(
                    data_type.instrument_id, 
                    1000,  # Default limit
                    correlation_id, 
                    start, 
                    end
                )
            elif data_type.type_name == "trade_ticks":
                await self._request_trade_ticks(
                    data_type.instrument_id, 
                    1000,  # Default limit
                    correlation_id, 
                    start, 
                    end
                )
            elif data_type.type_name == "bars":
                await self._request_bars(
                    data_type.bar_type, 
                    1000,  # Default limit
                    correlation_id, 
                    start, 
                    end
                )
            else:
                self._log.warning(f"Unsupported data type: {data_type.type_name}")
        except Exception as e:
            self._log.error(f"Failed to request data {data_type}: {e}")
            raise

    async def _subscribe_instrument_status(self, instrument_id: InstrumentId):
        self._log.info(f"Subscribing to instrument status for {instrument_id}")
        try:
            if self._connected and self._ws_client:
                symbol = str(instrument_id)
                # Subscribe to quotes to get status updates
                await self._ws_client.subscribe_quotes(symbol)
                self._log.debug(f"Instrument status subscription confirmed for {symbol}")
            else:
                self._log.warning("Not connected to MT5 WebSocket")
        except Exception as e:
            self._log.error(f"Failed to subscribe to instrument status for {instrument_id}: {e}")
            raise

    async def _subscribe_instrument_close(self, instrument_id: InstrumentId):
        self._log.info(f"Subscribing to instrument close for {instrument_id}")
        try:
            if self._connected and self._ws_client:
                symbol = str(instrument_id)
                # Subscribe to quotes to get close updates
                await self._ws_client.subscribe_quotes(symbol)
                self._log.debug(f"Instrument close subscription confirmed for {symbol}")
            else:
                self._log.warning("Not connected to MT5 WebSocket")
        except Exception as e:
            self._log.error(f"Failed to subscribe to instrument close for {instrument_id}: {e}")
            raise

    async def _request_instrument(self, instrument_id: InstrumentId, correlation_id: str, start: object = None, end: object = None):
        self._log.info(f"Requesting instrument: {instrument_id}")
        try:
            if not self._connected:
                self._log.warning("Not connected to MT5, cannot request instrument")
                return
                
            symbol = str(instrument_id)
            # Request instrument info via HTTP
            instrument_info = await self._http_client.get_symbol_info(symbol)
            
            # Convert to Nautilus instrument and publish
            # This would need proper conversion logic
            self._log.info(f"Received instrument info for {symbol}")
            
        except Exception as e:
            self._log.error(f"Failed to request instrument {instrument_id}: {e}")
            raise

    async def _request_quote_ticks(self, instrument_id: InstrumentId, limit: int, correlation_id: str, start: object = None, end: object = None):
        self._log.info(f"Requesting quote ticks for {instrument_id}")
        try:
            if not self._connected:
                self._log.warning("Not connected to MT5, cannot request quote ticks")
                return
                
            symbol = str(instrument_id)
            # Request historical rates (which include OHLC data)
            rates = await self._http_client.get_rates(symbol)
            
            # Convert to QuoteTick objects and publish to message bus
            self._log.info(f"Received {len(rates)} quote tick records for {symbol}")
            
            # Publish to message bus for processing
            # This would convert MT5 rates to Nautilus QuoteTick objects
            for rate in rates[:limit]:  # Limit the results
                # Convert and publish each tick
                pass
                
        except Exception as e:
            self._log.error(f"Failed to request quote ticks for {instrument_id}: {e}")
            raise

    async def _request_trade_ticks(self, instrument_id: InstrumentId, limit: int, correlation_id: str, start: object = None, end: object = None):
        self._log.info(f"Requesting trade ticks for {instrument_id}")
        try:
            if not self._connected:
                self._log.warning("Not connected to MT5, cannot request trade ticks")
                return
                
            symbol = str(instrument_id)
            # Request trade history
            trades = await self._http_client.get_trades()
            
            # Filter trades for the specific symbol
            symbol_trades = [t for t in trades if t.symbol == symbol][:limit]
            
            self._log.info(f"Received {len(symbol_trades)} trade tick records for {symbol}")
            
            # Convert to TradeTick objects and publish
            for trade in symbol_trades:
                # Convert and publish each trade
                pass
                
        except Exception as e:
            self._log.error(f"Failed to request trade ticks for {instrument_id}: {e}")
            raise

    async def _request_bars(self, bar_type: BarType, limit: int, correlation_id: str, start: object = None, end: object = None):
        self._log.info(f"Requesting bars for {bar_type}")
        try:
            if not self._connected:
                self._log.warning("Not connected to MT5, cannot request bars")
                return
                
            symbol = str(bar_type.instrument_id)
            # Request historical rates
            rates = await self._http_client.get_rates(symbol)
            
            # Convert to Bar objects based on bar_type timeframe
            self._log.info(f"Received {len(rates)} bar records for {symbol}")
            
            # Convert and publish bars
            for rate in rates[:limit]:  # Limit the results
                # Convert MT5 rate to Nautilus Bar and publish
                pass
                
        except Exception as e:
            self._log.error(f"Failed to request bars for {bar_type}: {e}")
            raise

    async def _request_order_book_snapshot(self, instrument_id: InstrumentId, correlation_id: str, start: object = None, end: object = None):
        self._log.info(f"Requesting order book snapshot for {instrument_id}")
        try:
            if not self._connected:
                self._log.warning("Not connected to MT5, cannot request order book snapshot")
                return
                
            symbol = str(instrument_id)
            # Order book data would come through WebSocket subscriptions
            # This is a placeholder for the API structure
            self._log.info(f"Order book snapshot requested for {symbol}")
            
        except Exception as e:
            self._log.error(f"Failed to request order book snapshot for {instrument_id}: {e}")
            raise
