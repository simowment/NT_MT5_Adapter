#!/usr/bin/env python3
"""
Backtest réel avec Nautilus Trader utilisant l'adaptateur MT5.

Ce script démontre comment :
1. Utiliser l'adaptateur MT5 pour récupérer les vraies données
2. Créer une stratégie de trading simple
3. Lancer un backtest avec les données réelles
4. Visualiser les résultats
"""

import asyncio
from datetime import datetime, timedelta
import pandas as pd

# Import de l'adaptateur MT5 (nécessite compilation)
from nautilus_trader.adapters.mt5 import (
    Mt5InstrumentProvider,
    Mt5InstrumentProviderConfig,
    Mt5HttpClient,
    Mt5WebSocketClient,
    Mt5Config,
    Mt5Credential
)

# Configuration MT5 réelle
MT5_CONFIG = {
    "host": "localhost",
    "port": 8080,
    "api_key": "demo_key",
    "api_secret": "demo_secret",
    "base_url": "http://localhost:8080",
    "ws_url": "ws://localhost:8080",
    "timeout": 30,
    "credentials": {
        "login": "your_login",  # Remplacer par vos identifiants réels
        "password": "your_password",
        "server": "your_server"
    }
}

# Stratégie de trading simple
class SimpleMovingAverageStrategy:
    """
    Stratégie de moyennes mobiles simple.
    Achète quand la MA rapide croise la MA lente vers le haut.
    Vend quand la MA rapide croise la MA lente vers le bas.
    """
    
    def __init__(self, fast_period: int = 10, slow_period: int = 30):
        self.fast_period = fast_period
        self.slow_period = slow_period
        self.name = "SimpleMA"
        
    async def on_start(self, engine):
        """Initialisation de la stratégie"""
        print(f"🚀 Démarrage de la stratégie {self.name}")
        self.engine = engine
        
    async def on_data(self, data):
        """Traitement des nouvelles données"""
        if hasattr(data, 'bar_type') and 'EURUSD' in str(data.bar_type):
            await self._process_bar(data)
            
    async def _process_bar(self, bar):
        """Traitement d'une barre de prix"""
        # Calculer les moyennes mobiles
        ma_fast = self._calculate_sma(bar.close, self.fast_period)
        ma_slow = self._calculate_sma(bar.close, self.slow_period)
        
        if ma_fast is None or ma_slow is None:
            return
            
        # Logique de trading
        if ma_fast > ma_slow and ma_fast <= ma_slow * 1.001:  # Croisement haussier
            await self._submit_buy_order(bar.symbol, 1.0, bar.close)
        elif ma_fast < ma_slow and ma_fast >= ma_slow * 0.999:  # Croisement baissier
            await self._submit_sell_order(bar.symbol, 1.0, bar.close)
            
    def _calculate_sma(self, prices, period):
        """Calcule une moyenne mobile simple"""
        if len(prices) < period:
            return None
        return sum(prices[-period:]) / period
        
    async def _submit_buy_order(self, symbol, quantity, price):
        """Soumet un ordre d'achat"""
        print(f"📈 BUY {symbol} x{quantity} @ {price:.5f}")
        # Dans un vrai backtest, on utiliserait : self.engine.submit_order(...)
        
    async def _submit_sell_order(self, symbol, quantity, price):
        """Soumet un ordre de vente"""
        print(f"📉 SELL {symbol} x{quantity} @ {price:.5f}")
        # Dans un vrai backtest, on utiliserait : self.engine.submit_order(...)

# Classe principale du backtest
class Mt5BacktestRunner:
    """Gestionnaire du backtest avec adaptateur MT5 réel"""
    
    def __init__(self):
        self.config = MT5_CONFIG
        self.strategy = SimpleMovingAverageStrategy()
        self.http_client = None
        self.instrument_provider = None
        
    async def create_mt5_http_client(self):
        """Crée le client HTTP MT5 réel"""
        try:
            config = Mt5Config(
                base_url=self.config["base_url"],
                ws_url=self.config["ws_url"],
                http_timeout=self.config["timeout"],
                ws_timeout=self.config["timeout"]
            )
            
            credential = Mt5Credential(
                login=self.config["credentials"]["login"],
                password=self.config["credentials"]["password"],
                server=self.config["credentials"]["server"]
            )
            
            return Mt5HttpClient(config, credential)
            
        except Exception as e:
            print(f"❌ Erreur création client HTTP: {e}")
            print("💡 L'adaptateur doit être compilé avec: build_mt5_adapter.sh")
            return None
        
    async def create_mt5_instrument_provider(self):
        """Crée le fournisseur d'instruments MT5 réel"""
        try:
            provider_config = Mt5InstrumentProviderConfig(
                host=self.config["host"],
                port=self.config["port"],
                base_url=self.config["base_url"],
                credentials={
                    "login": self.config["credentials"]["login"],
                    "password": self.config["credentials"]["password"],
                    "server": self.config["credentials"]["server"]
                }
            )
            
            return Mt5InstrumentProvider(config=provider_config)
            
        except Exception as e:
            print(f"❌ Erreur création provider: {e}")
            return None
        
    async def load_real_historical_data(self, symbol: str, timeframe: str = "1m", count: int = 1000):
        """Charge les données historiques réelles via l'adaptateur MT5"""
        print(f"📊 Chargement des données réelles {symbol} ({timeframe})...")
        
        if not self.http_client:
            self.http_client = await self.create_mt5_http_client()
            
        if not self.http_client:
            print("❌ Impossible de créer le client HTTP MT5")
            return None
            
        try:
            # 1. Se connecter et s'authentifier
            await self.http_client.login()
            print("✅ Connexion MT5 réussie")
            
            # 2. Récupérer les symboles disponibles
            symbols = await self.http_client.get_symbols()
            print(f"✅ Symboles disponibles: {len(symbols)}")
            
            # 3. Vérifier que le symbole existe
            if not any(hasattr(s, 'symbol') and s.symbol == symbol for s in symbols):
                print(f"⚠️  Symbole {symbol} non trouvé, utilisation du premier symbole disponible")
                if symbols:
                    symbol = symbols[0].symbol
                    print(f"✅ Utilisation de {symbol}")
                else:
                    print("❌ Aucun symbole disponible")
                    return None
            
            # 4. Charger les données historiques
            rates = await self.http_client.get_rates(symbol)
            print(f"✅ Données chargées: {len(rates)} barres pour {symbol}")
            
            if not rates:
                print("❌ Aucune donnée historique trouvée")
                return None
                
            # 5. Convertir en DataFrame
            data_list = []
            for rate in rates:
                data_list.append({
                    'timestamp': getattr(rate, 'timestamp', datetime.now()),
                    'open': getattr(rate, 'open', 0.0),
                    'high': getattr(rate, 'high', 0.0),
                    'low': getattr(rate, 'low', 0.0),
                    'close': getattr(rate, 'close', 0.0),
                    'volume': getattr(rate, 'volume', 0)
                })
            
            df = pd.DataFrame(data_list)
            df['timestamp'] = pd.to_datetime(df['timestamp'])
            df.set_index('timestamp', inplace=True)
            df = df.sort_index()  # S'assurer du tri chronologique
            
            return df
            
        except Exception as e:
            print(f"❌ Erreur lors du chargement: {e}")
            print("💡 Vérifiez que le serveur MT5 bridge fonctionne")
            return None
            
    async def get_mt5_account_info(self):
        """Récupère les informations de compte MT5 réelles"""
        if not self.http_client:
            self.http_client = await self.create_mt5_http_client()
            
        if not self.http_client:
            return None
            
        try:
            account_info = await self.http_client.get_account_info()
            print(f"✅ Compte MT5: {getattr(account_info, 'login', 'N/A')}")
            print(f"   - Balance: {getattr(account_info, 'balance', 'N/A')}")
            print(f"   - Equity: {getattr(account_info, 'equity', 'N/A')}")
            return account_info
            
        except Exception as e:
            print(f"❌ Erreur récupération compte: {e}")
            return None
        
    async def test_websocket_connection(self):
        """Test de connexion WebSocket réelle"""
        try:
            credential = Mt5Credential(
                login=self.config["credentials"]["login"],
                password=self.config["credentials"]["password"],
                server=self.config["credentials"]["server"]
            )
            
            ws_client = Mt5WebSocketClient(credential, self.config["ws_url"])
            await ws_client.connect()
            print("✅ Connexion WebSocket réussie")
            
            await ws_client.authenticate()
            print("✅ Authentification WebSocket réussie")
            
            await ws_client.subscribe("EURUSD")
            print("✅ Abonnement EURUSD créé")
            
            return True
            
        except Exception as e:
            print(f"❌ Erreur WebSocket: {e}")
            return False
        
    async def run_real_backtest(self, symbol: str = "EURUSD"):
        """Lance le backtest avec des données MT5 réelles"""
        print(f"🎯 Démarrage du backtest MT5 RÉEL - {symbol}")
        print("=" * 50)
        
        try:
            # 1. Récupérer les infos de compte
            account_info = await self.get_mt5_account_info()
            
            # 2. Charger les données réelles
            data = await self.load_real_historical_data(symbol)
            if data is None:
                return None
                
            print(f"✅ Données réelles chargées: {len(data)} barres")
            
            # 3. Afficher la qualité des données
            self._analyze_data_quality(data, symbol)
            
            # 4. Exécuter la stratégie sur les vraies données
            trades = await self._simulate_strategy(data, symbol)
            
            # 5. Calculer les métriques
            metrics = self._calculate_metrics(trades)
            
            # 6. Afficher les résultats
            self._display_results(symbol, metrics, trades, account_info)
            
            return metrics
            
        except Exception as e:
            print(f"❌ Erreur lors du backtest: {e}")
            return None
            
    def _analyze_data_quality(self, data, symbol):
        """Analyse la qualité des données réelles"""
        print(f"\n🔍 Analyse des données réelles {symbol}:")
        print(f"   - Période: {data.index[0]} → {data.index[-1]}")
        print(f"   - Barres: {len(data)}")
        print(f"   - Prix: {data['close'].iloc[0]:.5f} → {data['close'].iloc[-1]:.5f}")
        
        # Vérifier la continuité des données
        time_diffs = data.index.to_series().diff()
        max_gap = time_diffs.max()
        if max_gap > timedelta(hours=1):
            print(f"   - ⚠️  Trous détectés: {max_gap}")
        else:
            print("   - ✅ Données continues")
            
        # Statistiques de volume
        avg_volume = data['volume'].mean()
        print(f"   - Volume moyen: {avg_volume:.0f}")
        
    async def _simulate_strategy(self, data, symbol):
        """Simule l'exécution de la stratégie sur les vraies données"""
        trades = []
        position = 0  # 0 = pas de position, 1 = long, -1 = short
        
        # Calculer les moyennes mobiles
        data['ma_fast'] = data['close'].rolling(10).mean()
        data['ma_slow'] = data['close'].rolling(30).mean()
        
        for i in range(30, len(data)):  # Commencer après 30 barres pour avoir les MA
            current = data.iloc[i]
            previous = data.iloc[i-1]
            
            # Signaux de croisement
            fast_cross_up = (
                previous['ma_fast'] <= previous['ma_slow'] and 
                current['ma_fast'] > current['ma_slow']
            )
            fast_cross_down = (
                previous['ma_fast'] >= previous['ma_slow'] and 
                current['ma_fast'] < current['ma_slow']
            )
            
            if fast_cross_up and position <= 0:
                # Acheter
                if position < 0:
                    trades.append({
                        'type': 'CLOSE_SHORT',
                        'price': current['close'],
                        'timestamp': current.name,
                        'pnl': (trades[-1]['price'] - current['close']) * trades[-1]['size']
                    })
                
                trades.append({
                    'type': 'BUY',
                    'price': current['close'],
                    'timestamp': current.name,
                    'size': 1.0
                })
                position = 1
                
            elif fast_cross_down and position >= 0:
                # Vendre
                if position > 0:
                    trades.append({
                        'type': 'CLOSE_LONG',
                        'price': current['close'],
                        'timestamp': current.name,
                        'pnl': (current['close'] - trades[-1]['price']) * trades[-1]['size']
                    })
                
                trades.append({
                    'type': 'SELL',
                    'price': current['close'],
                    'timestamp': current.name,
                    'size': 1.0
                })
                position = -1
                
        # Fermer la position finale
        if position != 0 and trades:
            final_price = data['close'].iloc[-1]
            last_trade = trades[-1]
            if position > 0:
                trades.append({
                    'type': 'CLOSE_LONG',
                    'price': final_price,
                    'timestamp': data.index[-1],
                    'pnl': (final_price - last_trade['price']) * last_trade['size']
                })
            else:
                trades.append({
                    'type': 'CLOSE_SHORT',
                    'price': final_price,
                    'timestamp': data.index[-1],
                    'pnl': (last_trade['price'] - final_price) * last_trade['size']
                })
                
        return trades
        
    def _calculate_metrics(self, trades):
        """Calcule les métriques de performance"""
        if not trades:
            return {}
            
        # Calculer les PnL
        total_pnl = sum(trade.get('pnl', 0) for trade in trades)
        
        # Compter les trades gagnants/perdants
        winning_trades = [t for t in trades if t.get('pnl', 0) > 0]
        losing_trades = [t for t in trades if t.get('pnl', 0) < 0]
        
        # Calculer le win rate
        total_closed_trades = len(winning_trades) + len(losing_trades)
        win_rate = len(winning_trades) / total_closed_trades if total_closed_trades > 0 else 0
        
        # Calculer le profit factor
        gross_profit = sum(t.get('pnl', 0) for t in winning_trades)
        gross_loss = abs(sum(t.get('pnl', 0) for t in losing_trades))
        profit_factor = gross_profit / gross_loss if gross_loss > 0 else float('inf')
        
        return {
            'total_trades': total_closed_trades,
            'winning_trades': len(winning_trades),
            'losing_trades': len(losing_trades),
            'win_rate': win_rate,
            'total_pnl': total_pnl,
            'gross_profit': gross_profit,
            'gross_loss': gross_loss,
            'profit_factor': profit_factor
        }
        
    def _display_results(self, symbol, metrics, trades, account_info):
        """Affiche les résultats du backtest avec données réelles"""
        print("\n📈 RÉSULTATS DU BACKTEST MT5 RÉEL")
        print("=" * 50)
        print(f"Symbole: {symbol}")
        print(f"Stratégie: {self.strategy.name}")
        print("Données: RÉELLES via adaptateur MT5")
        
        if account_info:
            print(f"Compte: {getattr(account_info, 'login', 'N/A')}")
        
        if not metrics:
            print("❌ Aucun trade effectué")
            return
            
        print("\n📊 PERFORMANCE:")
        print(f"  Total des trades: {metrics['total_trades']}")
        print(f"  Trades gagnants: {metrics['winning_trades']}")
        print(f"  Trades perdants: {metrics['losing_trades']}")
        print(f"  Taux de réussite: {metrics['win_rate']:.1%}")
        print(f"  PnL total: {metrics['total_pnl']:.4f}")
        print(f"  Profit Factor: {metrics['profit_factor']:.2f}")
        
        print("\n💰 FINANCIER:")
        print(f"  Profit brut: {metrics['gross_profit']:.4f}")
        print(f"  Perte brute: {metrics['gross_loss']:.4f}")
        
        # Évaluation du résultat
        if metrics['total_pnl'] > 0:
            print("✅ Stratégie PROFITABLE sur données réelles")
        else:
            print("❌ Stratégie DÉFICITAIRE sur données réelles")
            
        if metrics['win_rate'] > 0.5:
            print("🎯 Bon taux de réussite")
        else:
            print("⚠️  Taux de réussite faible")

async def main():
    """Fonction principale"""
    print("🔄 Backtest MT5 RÉEL avec Nautilus Trader")
    print("=" * 60)
    print("Ce backtest utilise l'adaptateur MT5 pour charger")
    print("de VRAIES données de marché, pas de simulation.")
    print()
    
    # Créer le runner
    runner = Mt5BacktestRunner()
    
    # Test de connexion WebSocket (optionnel)
    print("🔌 Test de connexion WebSocket...")
    await runner.test_websocket_connection()
    
    # Tester différents symboles
    symbols = ["EURUSD", "GBPUSD", "USDJPY"]
    
    for symbol in symbols:
        print(f"\n🎯 Test pour {symbol} (données réelles)")
        try:
            metrics = await runner.run_real_backtest(symbol)
            if metrics:
                print(f"✅ Backtest {symbol} terminé avec succès")
            else:
                print(f"❌ Échec du backtest {symbol}")
        except Exception as e:
            print(f"❌ Erreur pour {symbol}: {e}")
            
    print("\n🎉 Tous les backtests réels sont terminés!")
    print("💡 Ce backtest utilise de vraies données MT5 via l'adaptateur")
    print("   - Données historiques réelles")
    print("   - Informations de compte réelles")
    print("   - WebSocket temps réel (si disponible)")

if __name__ == "__main__":
    # Exécuter le backtest réel
    asyncio.run(main())