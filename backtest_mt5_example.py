#!/usr/bin/env python3
"""
Exemple de backtest avec l'adaptateur MT5 et NautilusTrader
"""

import asyncio
import os
import sys
from datetime import datetime, timezone
from decimal import Decimal

# Ajouter le répertoire parent au path pour les imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from nautilus_trader.backtest.config import BacktestEngineConfig
from nautilus_trader.backtest.run import BacktestEngine
from nautilus_trader.model.data import BarSpecification
from nautilus_trader.model.instrument import InstrumentId
from nautilus_trader.examples.strategies.ema_cross import EmaCross
from mt5_config import data_config, execution_config, instrument_config

class MT5BacktestExample:
    """Exemple de backtest utilisant l'adaptateur MT5"""
    
    def __init__(self):
        self.instrument_id = InstrumentId.from_str("EURUSD.MT5")
        
    async def setup_backtest(self):
        """Configuration du moteur de backtest"""
        
        # Configuration du backtest
        config = BacktestEngineConfig(
            start_date=datetime(2024, 1, 1, tzinfo=timezone.utc),
            end_date=datetime(2024, 1, 31, tzinfo=timezone.utc),  # 1 mois de données
            data_client_configs=[data_config],
            execution_client_configs=[execution_config], 
            instrument_configs=[instrument_config],
            # Logging pour débogage
            logging_config={
                "log_level": "INFO",
                "log_file": "mt5_backtest.log"
            }
        )
        
        # Créer l'engine de backtest
        self.engine = BacktestEngine(config)
        
        print("✅ Moteur de backtest configuré")
        
    async def add_strategy(self):
        """Ajouter une stratégie au backtest"""
        
        # Stratégie EMA Cross simple
        self.strategy = EmaCross(
            instrument_id=self.instrument_id,
            bar_specification=BarSpecification.from_str("1-MINUTE-MID"),
            ema_period_fast=10,
            ema_period_slow=20,
            trading_fee_rate=Decimal("0.0002"),  # 2 pips
        )
        
        self.engine.add_strategy(self.strategy)
        print(f"✅ Stratégie EMA Cross ajoutée pour {self.instrument_id}")
        
    async def run_backtest(self):
        """Exécuter le backtest"""
        
        try:
            print("🚀 Démarrage du backtest...")
            print("📅 Période: 2024-01-01 à 2024-01-31")
            print(f"💱 Instrument: {self.instrument_id}")
            
            # Lancer le backtest
            result = await self.engine.run()
            
            # Afficher les résultats
            self.print_results(result)
            
            return result
            
        except Exception as e:
            print(f"❌ Erreur durant le backtest: {e}")
            raise
            
    def print_results(self, result):
        """Afficher les résultats du backtest"""
        
        print("\n" + "="*60)
        print("📊 RÉSULTATS DU BACKTEST MT5")
        print("="*60)
        
        # Résultats financiers
        net_pnl = result.portfolio.net_pnl()
        gross_pnl = result.portfolio.gross_pnl()
        total_fees = result.portfolio.total_fees()
        
        print(f"💰 Net PnL: ${net_pnl:.2f}")
        print(f"💰 Gross PnL: ${gross_pnl:.2f}")
        print(f"💸 Total Fees: ${total_fees:.2f}")
        
        # Statistiques de performance
        try:
            win_rate = result.performance.win_rate()
            profit_factor = result.performance.profit_factor()
            sharpe_ratio = result.performance.sharpe_ratio()
            max_drawdown = result.performance.max_drawdown()
            
            print(f"🎯 Win Rate: {win_rate:.1%}")
            print(f"📈 Profit Factor: {profit_factor:.2f}")
            print(f"📊 Sharpe Ratio: {sharpe_ratio:.2f}")
            print(f"📉 Max Drawdown: {max_drawdown:.1%}")
            
        except Exception as e:
            print(f"⚠️  Erreur lors du calcul des statistiques: {e}")
            
        # Statistiques de trading
        trades = result.trades
        print(f"🔄 Nombre de trades: {len(trades)}")
        
        if trades:
            winning_trades = [t for t in trades if t.pnl > 0]
            losing_trades = [t for t in trades if t.pnl < 0]
            
            print(f"✅ Trades gagnants: {len(winning_trades)}")
            print(f"❌ Trades perdants: {len(losing_trades)}")
            
            if winning_trades:
                avg_win = sum(t.pnl for t in winning_trades) / len(winning_trades)
                print(f"📈 Gain moyen: ${avg_win:.2f}")
                
            if losing_trades:
                avg_loss = sum(t.pnl for t in losing_trades) / len(losing_trades)
                print(f"📉 Perte moyenne: ${avg_loss:.2f}")
        
        print("="*60)
        
    async def cleanup(self):
        """Nettoyer les ressources"""
        if hasattr(self, 'engine'):
            await self.engine.dispose()
        print("🧹 Ressources nettoyées")

async def main():
    """Fonction principale"""
    
    print("🎯 Backtest MT5 avec NautilusTrader")
    print("-" * 40)
    
    # Vérifier les prérequis
    if not hasattr(sys, 'real_prefix') and not (hasattr(sys, 'base_prefix') and sys.base_prefix != sys.prefix):
        print("⚠️  Il est recommandé d'utiliser un environnement virtuel Python")
    
    # Créer et exécuter le backtest
    backtest = MT5BacktestExample()
    
    try:
        await backtest.setup_backtest()
        await backtest.add_strategy()
        result = await backtest.run_backtest()
        
        # Sauvegarder les résultats
        if result:
            result.to_csv("mt5_backtest_results.csv")
            print("💾 Résultats sauvegardés dans mt5_backtest_results.csv")
            
    except KeyboardInterrupt:
        print("\n⏹️  Backtest interrompu par l'utilisateur")
    except Exception as e:
        print(f"\n💥 Erreur fatale: {e}")
        import traceback
        traceback.print_exc()
    finally:
        await backtest.cleanup()

if __name__ == "__main__":
    print("Vérification des modules...")
    
    # Vérifier les modules requis
    required_modules = ['nautilus_trader']
    missing_modules = []
    
    for module in required_modules:
        try:
            __import__(module)
            print(f"✅ {module} disponible")
        except ImportError:
            missing_modules.append(module)
            print(f"❌ {module} manquant")
    
    if missing_modules:
        print("\nInstallez les modules manquants:")
        print(f"pip install {' '.join(missing_modules)}")
        sys.exit(1)
    
    # Lancer le backtest
    asyncio.run(main())