#!/usr/bin/env python3
"""
Test des imports MT5 adapter
"""

import sys
import os

# Ajouter le répertoire racine au PYTHONPATH
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

def test_imports():
    """Tester tous les imports MT5"""
    
    print("🔍 Test des imports MT5 adapter...")
    
    try:
        # Test import base
        from nautilus_trader.adapters.mt5.common import Mt5Credential
        print("✅ Mt5Credential importé avec succès")
        
        from nautilus_trader.adapters.mt5.config import Mt5DataClientConfig
        print("✅ Mt5DataClientConfig importé avec succès")
        
        from nautilus_trader.adapters.mt5.data import Mt5DataClient
        print("✅ Mt5DataClient importé avec succès")
        
        from nautilus_trader.adapters.mt5.execution import Mt5ExecutionClient
        print("✅ Mt5ExecutionClient importé avec succès")
        
        from nautilus_trader.adapters.mt5.factories import Mt5DataClientFactory
        print("✅ Mt5DataClientFactory importé avec succès")
        
        # Test création d'objets
        credential = Mt5Credential(
            login="123456",
            password="test",
            server="MetaQuotes-Demo"
        )
        print(f"✅ Mt5Credential créé: {credential}")
        
        config = Mt5DataClientConfig(
            base_url="http://localhost:8080",
            credential=credential
        )
        print(f"✅ Mt5DataClientConfig créé: {config}")
        
        return True
        
    except ImportError as e:
        print(f"❌ Erreur d'import: {e}")
        return False
    except Exception as e:
        print(f"❌ Erreur générale: {e}")
        return False

if __name__ == "__main__":
    success = test_imports()
    
    if success:
        print("\n🎉 Tous les imports fonctionnent !")
        print("\n📋 Prochaines étapes:")
        print("1. Construire l'adaptateur Rust: cargo build -p nautilus-adapters-mt5 --features python-bindings --release")
        print("2. Copier la bibliothèque .pyd/.so dans nautilus_trader/adapters/mt5/bindings/")
        print("3. Lancer le backtest: python backtest_mt5_example.py")
    else:
        print("\n❌ Il y a des problèmes d'import")