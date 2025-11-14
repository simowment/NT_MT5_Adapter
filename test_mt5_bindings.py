#!/usr/bin/env python3
"""
Test script pour vérifier les bindings Python de l'adaptateur MT5
"""

import sys
import traceback

def test_imports():
    """Test imports pour vérifier les bindings Python"""
    print("=== Test des imports MT5 NautilusTrader ===")
    
    try:
        # Test d'import des modules de base
        print("Test 1: Import de nautilus_adapters_mt5...")
        import nautilus_trader.adapters.mt5 as nautilus_adapters_mt5
        print(f"✓ nautilus_adapters_mt5 importé avec succès")
        print(f"  Version: {getattr(nautilus_adapters_mt5, '__version__', 'Non définie')}")
        
        # Test d'accès aux classes de configuration
        print("\nTest 2: Accès aux classes de configuration...")
        try:
            Mt5Config = nautilus_adapters_mt5.Mt5Config
            print("✓ Mt5Config accessible")
        except AttributeError as e:
            print(f"✗ Mt5Config non accessible: {e}")
            
        # Test d'accès aux clients
        print("\nTest 3: Accès aux clients MT5...")
        try:
            Mt5DataClient = nautilus_adapters_mt5.Mt5DataClient
            print("✓ Mt5DataClient accessible")
        except AttributeError as e:
            print(f"✗ Mt5DataClient non accessible: {e}")
            
        try:
            Mt5ExecutionClient = nautilus_adapters_mt5.Mt5ExecutionClient
            print("✓ Mt5ExecutionClient accessible")
        except AttributeError as e:
            print(f"✗ Mt5ExecutionClient non accessible: {e}")
            
        try:
            Mt5InstrumentProvider = nautilus_adapters_mt5.Mt5InstrumentProvider
            print("✓ Mt5InstrumentProvider accessible")
        except AttributeError as e:
            print(f"✗ Mt5InstrumentProvider non accessible: {e}")
            
        # Test d'accès aux modèles HTTP
        print("\nTest 4: Accès aux modèles HTTP...")
        try:
            Mt5Symbol = nautilus_adapters_mt5.Mt5Symbol
            print("✓ Mt5Symbol accessible")
        except AttributeError as e:
            print(f"✗ Mt5Symbol non accessible: {e}")
            
        try:
            Mt5Rate = nautilus_adapters_mt5.Mt5Rate
            print("✓ Mt5Rate accessible")
        except AttributeError as e:
            print(f"✗ Mt5Rate non accessible: {e}")
            
        # Test des listes d'attributs
        print("\nTest 5: Liste de tous les attributs disponibles:")
        attrs = dir(nautilus_adapters_mt5)
        print(f"  Total: {len(attrs)} attributs")
        
        # Grouper les attributs par type
        classes = [attr for attr in attrs if not attr.startswith('_')]
        print(f"  Classes/méthodes publiques: {len(classes)}")
        
        if classes:
            print("  Premiers attributs disponibles:")
            for attr in classes[:10]:
                print(f"    - {attr}")
            if len(classes) > 10:
                print(f"    ... et {len(classes) - 10} autres")
        else:
            print("  ⚠️  Aucun attribut public trouvé")
            
        print("\n=== Résumé du test ===")
        if len(classes) > 20:
            print("✓ Succès: Beaucoup d'attributs disponibles - bindings probablement fonctionnels")
        elif len(classes) > 5:
            print("⚠️  Partiel: Quelques attributs disponibles - bindings partiellement fonctionnels")
        else:
            print("✗ Échec: Peu d'attributs disponibles - bindings non fonctionnels")
            
        return len(classes)
        
    except ImportError as e:
        print(f"✗ Échec import nautilus_adapters_mt5: {e}")
        print("\nDétails de l'erreur:")
        traceback.print_exc()
        return 0
    except Exception as e:
        print(f"✗ Erreur inattendue: {e}")
        traceback.print_exc()
        return 0

def main():
    """Fonction principale"""
    print("Test des bindings Python MT5 pour NautilusTrader")
    print("=" * 50)
    
    # Test des imports
    attr_count = test_imports()
    
    # Code de retour
    if attr_count > 20:
        print(f"\n🎉 SUCCÈS! {attr_count} attributs trouvés - Les bindings semblent fonctionnels")
        return 0
    elif attr_count > 5:
        print(f"\n⚠️  SUCCÈS PARTIEL! {attr_count} attributs trouvés - Nécessite investigation")
        return 1
    else:
        print(f"\n❌ ÉCHEC! Seulement {attr_count} attributs trouvés - Les bindings ne sont pas fonctionnels")
        return 2

if __name__ == "__main__":
    sys.exit(main())