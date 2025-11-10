# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# -------------------------------------------------------------------------------------------------

"""
Tests d'intégration pour l'adaptateur MetaTrader 5.

Ces tests valident que tous les composants Python fonctionnent correctement ensemble :
- Mt5InstrumentProvider avec découverte d'instruments
- Mt5DataClient avec souscriptions et requêtes
- Mt5ExecutionClient avec gestion d'ordres
- Gestion d'erreurs et logging
"""

import pytest
import asyncio
from unittest.mock import Mock, patch, AsyncMock
from typing import List, Dict, Any

# Imports des composants MT5
from nautilus_trader.adapters.mt5.config import (
    Mt5InstrumentProviderConfig,
    Mt5DataClientConfig,
    Mt5ExecClientConfig
)
from nautilus_trader.adapters.mt5.data import Mt5DataClient
from nautilus_trader.adapters.mt5.execution import Mt5ExecutionClient
from nautilus_trader.adapters.mt5.providers import Mt5InstrumentProvider
from nautilus_trader.adapters.mt5.factories import Mt5Factories

# Imports Nautilus pour les tests
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock, MessageBus
from nautilus_trader.model.identifiers import InstrumentId, ClientId
from nautilus_trader.model.data import BarType
from nautilus_trader.execution.messages import SubmitOrder
from nautilus_trader.model.objects import OrderSide, OrderType, Quantity

# Imports pour gestion d'erreurs MT5
from nautilus_trader.adapters.mt5.data import (
    Mt5DataError, 
    Mt5ConnectionError, 
    Mt5SubscriptionError,
    Mt5DataRequestError,
    Mt5ParsingError
)


class TestMt5Integration:
    """Tests d'intégration complets pour l'adaptateur MT5."""

    @pytest.fixture
    def event_loop(self):
        """Créer un event loop pour les tests async."""
        loop = asyncio.new_event_loop()
        yield loop
        loop.close()

    @pytest.fixture
    def mt5_configs(self):
        """Configuration MT5 pour les tests."""
        return {
            'instrument_provider': Mt5InstrumentProviderConfig(
                mt5_host="localhost",
                mt5_port=8080,
                mt5_login="test_user",
                mt5_password="test_pass",
                mt5_server="test_server",
                filter_currencies=["USD", "EUR"],
                filter_indices=["US30"],
                filter_cfds=True,
                filter_futures=False,
                auto_discover_instruments=True,
                cache_expiry=300,
                enable_logging=True
            ),
            'data_client': Mt5DataClientConfig(
                mt5_host="localhost",
                mt5_port=8080,
                mt5_login="test_user",
                mt5_password="test_pass",
                mt5_server="test_server",
                subscribe_quotes=True,
                subscribe_trades=True,
                subscribe_order_book=False,
                max_subscriptions=100,
                connection_retry_attempts=3,
                connection_retry_delay=1,
                heartbeat_interval=30,
                reconnection_enabled=True,
                enable_logging=True
            ),
            'execution_client': Mt5ExecClientConfig(
                mt5_host="localhost",
                mt5_port=8080,
                mt5_login="test_user",
                mt5_password="test_pass",
                mt5_server="test_server",
                max_concurrent_orders=50,
                order_timeout=30,
                connection_retry_attempts=3,
                connection_retry_delay=1,
                enable_partial_fills=True,
                enable_market_data=True,
                risk_management_enabled=True,
                position_sizing_enabled=True,
                simulate_orders=True,  # Mode simulation pour tests
                enable_logging=True
            )
        }

    @pytest.fixture
    def nautilus_components(self):
        """Composants Nautilus pour les tests."""
        return {
            'msgbus': MessageBus(),
            'cache': Cache(),
            'clock': LiveClock(),
            'loop': asyncio.new_event_loop()
        }

    @pytest.fixture
    def mock_http_client(self):
        """Mock du client HTTP MT5."""
        client = Mock()
        client.login = AsyncMock(return_value=None)
        client.get_account_info = AsyncMock(return_value=Mock(
            login="123456",
            balance=10000.0,
            equity=10000.0,
            margin=0.0,
            margin_free=10000.0,
            margin_level=0.0
        ))
        client.get_symbols = AsyncMock(return_value=[
            Mock(
                symbol="EURUSD",
                digits=5,
                point_size=0.00001,
                volume_min=0.01,
                volume_max=100.0,
                volume_step=0.01,
                contract_size=100000.0,
                margin_initial=0.03,
                margin_maintenance=0.03
            ),
            Mock(
                symbol="US30",
                digits=2,
                point_size=0.01,
                volume_min=0.01,
                volume_max=100.0,
                volume_step=0.01,
                contract_size=100.0,
                margin_initial=0.01,
                margin_maintenance=0.01
            )
        ])
        client.get_symbol_info = AsyncMock(return_value=Mock(
            symbol="EURUSD",
            digits=5,
            point_size=0.00001,
            volume_min=0.01,
            volume_max=100.0,
            volume_step=0.01,
            contract_size=100000.0,
            margin_initial=0.03,
            margin_maintenance=0.03
        ))
        client.get_rates = AsyncMock(return_value=[
            Mock(
                symbol="EURUSD",
                time=1640995200,
                open=1.12345,
                high=1.12400,
                low=1.12300,
                close=1.12350,
                tick_volume=100
            )
        ])
        client.get_positions = AsyncMock(return_value=[])
        client.get_trades = AsyncMock(return_value=[])
        client.get_orders = AsyncMock(return_value=[])
        return client

    @pytest.fixture
    def mock_ws_client(self):
        """Mock du client WebSocket MT5."""
        client = Mock()
        client.connect = AsyncMock(return_value=None)
        client.authenticate = AsyncMock(return_value=None)
        client.disconnect = AsyncMock(return_value=None)
        client.subscribe_quotes = AsyncMock(return_value=None)
        client.subscribe_trades = AsyncMock(return_value=None)
        client.unsubscribe = AsyncMock(return_value=None)
        return client

    # Tests d'intégration Instrument Provider

    @pytest.mark.asyncio
    async def test_instrument_provider_initialization(self, mt5_configs, nautilus_components):
        """Test d'initialisation du Mt5InstrumentProvider."""
        with patch('nautilus_trader.adapters.mt5.providers.Mt5HttpClient') as mock_http:
            with patch('nautilus_trader.adapters.mt5.providers.Mt5WebSocketClient') as mock_ws:
                # Configuration des mocks
                mock_http_instance = Mock()
                mock_ws_instance = Mock()
                mock_http_instance.login = AsyncMock(return_value=None)
                mock_http_instance.get_symbols = AsyncMock(return_value=[])
                mock_http.return_value = mock_http_instance
                mock_ws.return_value = mock_ws_instance

                # Test d'initialisation
                provider = Mt5InstrumentProvider(
                    config=mt5_configs['instrument_provider'],
                    msgbus=nautilus_components['msgbus'],
                    cache=nautilus_components['cache'],
                    clock=nautilus_components['clock']
                )

                assert provider is not None
                assert provider.config == mt5_configs['instrument_provider']
                assert provider.msgbus == nautilus_components['msgbus']
                assert provider.cache == nautilus_components['cache']
                assert provider.clock == nautilus_components['clock']

    @pytest.mark.asyncio
    async def test_instrument_provider_load_instruments(self, mt5_configs, nautilus_components, mock_http_client):
        """Test de chargement d'instruments via Mt5InstrumentProvider."""
        with patch('nautilus_trader.adapters.mt5.providers.Mt5HttpClient') as mock_http:
            mock_http.return_value = mock_http_client

            provider = Mt5InstrumentProvider(
                config=mt5_configs['instrument_provider'],
                msgbus=nautilus_components['msgbus'],
                cache=nautilus_components['cache'],
                clock=nautilus_components['clock']
            )

            # Connexion
            await provider.load_instruments()
            
            # Vérifications
            mock_http_client.login.assert_called_once()
            mock_http_client.get_symbols.assert_called_once()
            
            # Test des filtres
            assert len(provider.config.filter_currencies) == 2
            assert "USD" in provider.config.filter_currencies
            assert "EUR" in provider.config.filter_currencies

    # Tests d'intégration Data Client

    @pytest.mark.asyncio
    async def test_data_client_initialization(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test d'initialisation du Mt5DataClient."""
        data_client = Mt5DataClient(
            loop=nautilus_components['loop'],
            http_client=mock_http_client,
            ws_client=mock_ws_client,
            msgbus=nautilus_components['msgbus'],
            cache=nautilus_components['cache'],
            clock=nautilus_components['clock']
        )

        assert data_client is not None
        assert data_client._http_client == mock_http_client
        assert data_client._ws_client == mock_ws_client
        assert data_client._connected == False

    @pytest.mark.asyncio
    async def test_data_client_connection(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test de connexion du Mt5DataClient."""
        data_client = Mt5DataClient(
            loop=nautilus_components['loop'],
            http_client=mock_http_client,
            ws_client=mock_ws_client,
            msgbus=nautilus_components['msgbus'],
            cache=nautilus_components['cache'],
            clock=nautilus_components['clock']
        )

        # Test de connexion
        data_client.connect()
        
        # Vérifications des appels de connexion
        mock_http_client.login.assert_called_once()
        mock_ws_client.connect.assert_called_once()
        mock_ws_client.authenticate.assert_called_once()
        
        assert data_client._connected == True

    @pytest.mark.asyncio
    async def test_data_client_disconnection(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test de déconnexion du Mt5DataClient."""
        data_client = Mt5DataClient(
            loop=nautilus_components['loop'],
            http_client=mock_http_client,
            ws_client=mock_ws_client,
            msgbus=nautilus_components['msgbus'],
            cache=nautilus_components['cache'],
            clock=nautilus_components['clock']
        )

        # Connexion puis déconnexion
        data_client.connect()
        data_client.disconnect()
        
        # Vérifications
        mock_ws_client.disconnect.assert_called_once()
        assert data_client._connected == False

    @pytest.mark.asyncio
    async def test_data_client_subscriptions(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test des souscriptions du Mt5DataClient."""
        data_client = Mt5DataClient(
            loop=nautilus_components['loop'],
            http_client=mock_http_client,
            ws_client=mock_ws_client,
            msgbus=nautilus_components['msgbus'],
            cache=nautilus_components['cache'],
            clock=nautilus_components['clock']
        )

        # Connexion
        data_client.connect()

        # Test de souscriptions
        instrument_id = InstrumentId.from_str("EURUSD.MT5")
        
        # Quote ticks
        await data_client._subscribe_quote_ticks(instrument_id)
        mock_ws_client.subscribe_quotes.assert_called_with("EURUSD.MT5")
        
        # Trade ticks
        await data_client._subscribe_trade_ticks(instrument_id)
        mock_ws_client.subscribe_trades.assert_called_with("EURUSD.MT5")
        
        # Bars
        bar_type = BarType.from_str("EURUSD.MT5-1m-BID-EXTERNAL")
        await data_client._subscribe_bars(bar_type)
        mock_ws_client.subscribe_quotes.assert_called_with("EURUSD.MT5")

    @pytest.mark.asyncio
    async def test_data_client_requests(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test des requêtes de données du Mt5DataClient."""
        data_client = Mt5DataClient(
            loop=nautilus_components['loop'],
            http_client=mock_http_client,
            ws_client=mock_ws_client,
            msgbus=nautilus_components['msgbus'],
            cache=nautilus_components['cache'],
            clock=nautilus_components['clock']
        )

        # Connexion
        data_client.connect()

        # Test de requêtes
        instrument_id = InstrumentId.from_str("EURUSD.MT5")
        
        # Quote ticks
        await data_client._request_quote_ticks(instrument_id, 100, "test_correlation")
        mock_http_client.get_rates.assert_called_with("EURUSD.MT5")
        
        # Trade ticks
        await data_client._request_trade_ticks(instrument_id, 100, "test_correlation")
        mock_http_client.get_trades.assert_called_once()
        
        # Bars
        bar_type = BarType.from_str("EURUSD.MT5-1m-BID-EXTERNAL")
        await data_client._request_bars(bar_type, 100, "test_correlation")
        mock_http_client.get_rates.assert_called_with("EURUSD.MT5")

    # Tests d'intégration Execution Client

    @pytest.mark.asyncio
    async def test_execution_client_initialization(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test d'initialisation du Mt5ExecutionClient."""
        exec_client = Mt5ExecutionClient(
            loop=nautilus_components['loop'],
            http_client=mock_http_client,
            ws_client=mock_ws_client,
            msgbus=nautilus_components['msgbus'],
            cache=nautilus_components['cache'],
            clock=nautilus_components['clock']
        )

        assert exec_client is not None
        assert exec_client._http_client == mock_http_client
        assert exec_client._ws_client == mock_ws_client
        assert exec_client._connected == False

    @pytest.mark.asyncio
    async def test_execution_client_order_operations(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test des opérations d'ordres du Mt5ExecutionClient."""
        exec_client = Mt5ExecutionClient(
            loop=nautilus_components['loop'],
            http_client=mock_http_client,
            ws_client=mock_ws_client,
            msgbus=nautilus_components['msgbus'],
            cache=nautilus_components['cache'],
            clock=nautilus_components['clock']
        )

        # Connexion
        exec_client.connect()

        # Configuration mock pour ordre
        mock_http_client.submit_order = AsyncMock(return_value=Mock(
            order_id=12345,
            symbol="EURUSD",
            volume=1.0,
            price=1.08,
            order_type="BUY",
            status="EXECUTED"
        ))

        # Test soumission d'ordre
        order = SubmitOrder(
            instrument_id=InstrumentId.from_str("EURUSD.MT5"),
            client_order_id=ClientId("test_order_001"),
            order_side=OrderSide.BUY,
            order_type=OrderType.MARKET,
            quantity=Quantity.from_str("1.0"),
        )

        await exec_client._submit_order(order)
        
        # Vérifications
        mock_http_client.submit_order.assert_called_once()
        
        # Test d'annulation
        exec_client._cancel_order(order)
        # Note: Cette méthode n'est pas async, donc on ne peut pas l'await
        
        # Test de modification
        exec_client._modify_order(order)
        # Note: Cette méthode n'est pas async, donc on ne peut pas l'await

    # Tests de gestion d'erreurs

    @pytest.mark.asyncio
    async def test_data_client_error_handling(self, mt5_configs, nautilus_components):
        """Test de la gestion d'erreurs du Mt5DataClient."""
        # Client avec connexion qui échoue
        mock_http_client = Mock()
        mock_http_client.login = AsyncMock(side_effect=Exception("Connection failed"))
        
        mock_ws_client = Mock()
        mock_ws_client.connect = AsyncMock(side_effect=Exception("WebSocket failed"))
        mock_ws_client.authenticate = AsyncMock(side_effect=Exception("Auth failed"))

        data_client = Mt5DataClient(
            loop=nautilus_components['loop'],
            http_client=mock_http_client,
            ws_client=mock_ws_client,
            msgbus=nautilus_components['msgbus'],
            cache=nautilus_components['cache'],
            clock=nautilus_components['clock']
        )

        # Test de connexion qui échoue
        with pytest.raises(Exception, match="Connection failed"):
            data_client.connect()

        # Test de subscription qui échoue (non connecté)
        data_client._connected = False
        instrument_id = InstrumentId.from_str("EURUSD.MT5")
        
        # Ne devrait pas lever d'exception, mais logger un warning
        await data_client._subscribe_quote_ticks(instrument_id)
        # Le test passe si aucune exception n'est levée

    @pytest.mark.asyncio
    async def test_mt5_exceptions_hierarchy(self):
        """Test de la hiérarchie des exceptions MT5."""
        # Test que les exceptions sont bien définies
        assert issubclass(Mt5DataError, Exception)
        assert issubclass(Mt5ConnectionError, Mt5DataError)
        assert issubclass(Mt5SubscriptionError, Mt5DataError)
        assert issubclass(Mt5DataRequestError, Mt5DataError)
        assert issubclass(Mt5ParsingError, Mt5DataError)
        
        # Test de création d'exceptions
        try:
            raise Mt5ConnectionError("Test connection error")
        except Mt5DataError as e:
            assert str(e) == "Test connection error"
            assert isinstance(e, Mt5ConnectionError)
            assert isinstance(e, Mt5DataError)

    # Tests d'intégration des factories

    @pytest.mark.asyncio
    async def test_mt5_factories_integration(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test d'intégration des Mt5Factories."""
        with patch('nautilus_trader.adapters.mt5.factories.Mt5HttpClient') as mock_http:
            with patch('nautilus_trader.adapters.mt5.factories.Mt5WebSocketClient') as mock_ws:
                # Configuration des mocks
                mock_http.return_value = mock_http_client
                mock_ws.return_value = mock_ws_client

                # Test création des clients via factories
                http_client = Mt5Factories.create_http_client(
                    mt5_configs['data_client']
                )
                ws_client = Mt5Factories.create_ws_client(
                    mt5_configs['data_client']
                )
                instrument_provider = Mt5Factories.create_instrument_provider(
                    mt5_configs['instrument_provider'],
                    nautilus_components['msgbus'],
                    nautilus_components['cache'],
                    nautilus_components['clock']
                )
                data_client = Mt5Factories.create_data_client(
                    mt5_configs['data_client'],
                    http_client,
                    ws_client,
                    nautilus_components['msgbus'],
                    nautilus_components['cache'],
                    nautilus_components['clock'],
                    nautilus_components['loop']
                )
                exec_client = Mt5Factories.create_execution_client(
                    mt5_configs['execution_client'],
                    http_client,
                    ws_client,
                    nautilus_components['msgbus'],
                    nautilus_components['cache'],
                    nautilus_components['clock'],
                    nautilus_components['loop']
                )

                # Vérifications
                assert http_client is not None
                assert ws_client is not None
                assert instrument_provider is not None
                assert data_client is not None
                assert exec_client is not None

    # Tests de configuration

    def test_configurations_validation(self, mt5_configs):
        """Test de validation des configurations."""
        # Test configuration Instrument Provider
        config = mt5_configs['instrument_provider']
        assert config.mt5_host == "localhost"
        assert config.mt5_port == 8080
        assert config.mt5_login == "test_user"
        assert config.mt5_password == "test_pass"
        assert config.mt5_server == "test_server"
        assert config.filter_currencies == ["USD", "EUR"]
        assert config.filter_indices == ["US30"]
        assert config.filter_cfds == True
        assert config.filter_futures == False
        assert config.auto_discover_instruments == True
        assert config.cache_expiry == 300
        assert config.enable_logging == True

        # Test configuration Data Client
        config = mt5_configs['data_client']
        assert config.subscribe_quotes == True
        assert config.subscribe_trades == True
        assert config.subscribe_order_book == False
        assert config.max_subscriptions == 100
        assert config.connection_retry_attempts == 3
        assert config.connection_retry_delay == 1
        assert config.heartbeat_interval == 30
        assert config.reconnection_enabled == True
        assert config.enable_logging == True

        # Test configuration Execution Client
        config = mt5_configs['execution_client']
        assert config.max_concurrent_orders == 50
        assert config.order_timeout == 30
        assert config.connection_retry_attempts == 3
        assert config.connection_retry_delay == 1
        assert config.enable_partial_fills == True
        assert config.enable_market_data == True
        assert config.risk_management_enabled == True
        assert config.position_sizing_enabled == True
        assert config.simulate_orders == True  # Mode simulation pour tests
        assert config.enable_logging == True

    # Tests de logging

    @pytest.mark.asyncio
    async def test_logging_functionality(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test de la fonctionnalité de logging."""
        data_client = Mt5DataClient(
            loop=nautilus_components['loop'],
            http_client=mock_http_client,
            ws_client=mock_ws_client,
            msgbus=nautilus_components['msgbus'],
            cache=nautilus_components['cache'],
            clock=nautilus_components['clock']
        )

        # Test que le logging est configuré
        assert hasattr(data_client, '_log')
        assert data_client._log is not None

        # Test des méthodes de logging
        with patch.object(data_client._log, 'info') as mock_log_info:
            data_client.connect()
            # Vérifier que le logging d'information a été appelé
            mock_log_info.assert_called()

    # Tests de performance

    @pytest.mark.asyncio
    async def test_performance_multiple_subscriptions(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test de performance avec de nombreuses souscriptions."""
        data_client = Mt5DataClient(
            loop=nautilus_components['loop'],
            http_client=mock_http_client,
            ws_client=mock_ws_client,
            msgbus=nautilus_components['msgbus'],
            cache=nautilus_components['cache'],
            clock=nautilus_components['clock']
        )

        # Connexion
        data_client.connect()

        # Test avec de nombreux instruments
        symbols = [f"SYMBOL{i:03d}.MT5" for i in range(50)]
        
        for symbol in symbols:
            instrument_id = InstrumentId.from_str(symbol)
            await data_client._subscribe_quote_ticks(instrument_id)

        # Vérifications
        assert mock_ws_client.subscribe_quotes.call_count == 50

    # Tests d'intégration bout en bout

    @pytest.mark.asyncio
    async def test_end_to_end_integration(self, mt5_configs, nautilus_components, mock_http_client, mock_ws_client):
        """Test d'intégration bout en bout complet."""
        with patch('nautilus_trader.adapters.mt5.factories.Mt5HttpClient') as mock_http:
            with patch('nautilus_trader.adapters.mt5.factories.Mt5WebSocketClient') as mock_ws:
                # Configuration des mocks
                mock_http.return_value = mock_http_client
                mock_ws.return_value = mock_ws_client

                # Création de tous les composants
                instrument_provider = Mt5Factories.create_instrument_provider(
                    mt5_configs['instrument_provider'],
                    nautilus_components['msgbus'],
                    nautilus_components['cache'],
                    nautilus_components['clock']
                )
                http_client = Mt5Factories.create_http_client(mt5_configs['data_client'])
                ws_client = Mt5Factories.create_ws_client(mt5_configs['data_client'])
                data_client = Mt5Factories.create_data_client(
                    mt5_configs['data_client'],
                    http_client,
                    ws_client,
                    nautilus_components['msgbus'],
                    nautilus_components['cache'],
                    nautilus_components['clock'],
                    nautilus_components['loop']
                )
                exec_client = Mt5Factories.create_execution_client(
                    mt5_configs['execution_client'],
                    http_client,
                    ws_client,
                    nautilus_components['msgbus'],
                    nautilus_components['cache'],
                    nautilus_components['clock'],
                    nautilus_components['loop']
                )

                # Séquence d'utilisation normale
                
                # 1. Chargement d'instruments
                await instrument_provider.load_instruments()
                
                # 2. Connexion des clients
                data_client.connect()
                exec_client.connect()
                
                # 3. Souscriptions de données
                instrument_id = InstrumentId.from_str("EURUSD.MT5")
                await data_client._subscribe_quote_ticks(instrument_id)
                await data_client._subscribe_trade_ticks(instrument_id)
                
                # 4. Requêtes de données
                await data_client._request_quote_ticks(instrument_id, 100, "test_request")
                await data_client._request_trade_ticks(instrument_id, 100, "test_request")
                
                # 5. Soumission d'ordre
                order = SubmitOrder(
                    instrument_id=instrument_id,
                    client_order_id=ClientId("e2e_test_order"),
                    order_side=OrderSide.BUY,
                    order_type=OrderType.MARKET,
                    quantity=Quantity.from_str("1.0"),
                )
                await exec_client._submit_order(order)
                
                # Vérifications globales
                mock_http_client.login.assert_called()
                mock_ws_client.connect.assert_called()
                mock_ws_client.authenticate.assert_called()
                assert data_client._connected == True
                assert exec_client._connected == True


if __name__ == "__main__":
    # Exécution des tests en mode standalone
    pytest.main([__file__, "-v"])