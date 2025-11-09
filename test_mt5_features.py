#!/usr/bin/env python3
"""
Test script pour vérifier les fonctionnalités MT5 déjà implémentées.

Ce script teste:
1. Les parseurs Rust (timestamps, instruments, prix)
2. La création de client HTTP
3. La logique d'authentification
4. La validation des configurations
"""

import subprocess
import sys
import os

def run_cargo_test():
    """Exécute les tests Rust pour le module MT5"""
    print("🔧 Compilation et exécution des tests Rust...")
    
    try:
        # Change to the MT5 adapter directory
        os.chdir("crates/adapters/mt5")
        
        # Run cargo test
        result = subprocess.run(
            ["cargo", "test", "--", "--nocapture"],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        print("STDOUT:")
        print(result.stdout)
        
        if result.stderr:
            print("STDERR:")
            print(result.stderr)
            
        if result.returncode == 0:
            print("✅ Tous les tests Rust sont passés avec succès!")
        else:
            print(f"❌ Échec des tests Rust: code de retour {result.returncode}")
            
    except subprocess.TimeoutExpired:
        print("⏰ Timeout lors de l'exécution des tests")
    except FileNotFoundError:
        print("❌ Cargo non trouvé. Assurez-vous que Rust est installé.")
    except Exception as e:
        print(f"❌ Erreur lors de l'exécution des tests: {e}")

def check_dependencies():
    """Vérifie les dépendances nécessaires"""
    print("🔍 Vérification des dépendances...")
    
    # Check if we're in the right directory
    if not os.path.exists("Cargo.toml"):
        print("❌ Pas de Cargo.toml trouvé. Assurez-vous d'être dans le répertoire du projet.")
        return False
        
    # Check if we can see the MT5 source files (from project root)
    required_files = [
        "crates/adapters/mt5/src/common/parse.rs",
        "crates/adapters/mt5/src/http/client.rs",
        "crates/adapters/mt5/src/websocket/client.rs",
        "crates/adapters/mt5/src/python/bindings.rs"
    ]
    
    missing_files = []
    for file_path in required_files:
        if not os.path.exists(file_path):
            missing_files.append(file_path)
    
    if missing_files:
        print(f"❌ Fichiers manquants: {missing_files}")
        return False
    
    print("✅ Fichiers sources présents")
    return True

def show_testable_features():
    """Affiche les fonctionnalités testables"""
    print("\n🎯 Fonctionnalités testables actuellement:")
    print("1. ✅ Parseurs d'instruments (FX, CFD, Futures)")
    print("2. ✅ Parseurs de timestamps MT5")
    print("3. ✅ Parseurs de prix avec précision")
    print("4. ✅ Parseurs de volumes")
    print("5. ✅ Client HTTP (création, configuration)")
    print("6. ✅ Taxonomie d'erreurs HTTP")
    print("7. ✅ Méthodes d'authentification")
    print("8. ✅ Gestion des tokens")
    print("9. ✅ Client WebSocket (structure de base)")
    print("10. ✅ Bindings PyO3 (structures)")
    
    print("\n📝 Tests disponibles:")
    print("- Tests unitaires des parseurs (common/parse.rs)")
    print("- Tests d'intégration HTTP avec wiremock")
    print("- Tests de création de client")
    print("- Tests d'authentification")
    print("- Tests de gestion d'erreurs")

def main():
    print("🚀 Test des fonctionnalités MT5 adapter")
    print("=" * 50)
    
    # Show what we can test
    show_testable_features()
    
    print("\n" + "=" * 50)
    
    # Check dependencies
    if not check_dependencies():
        sys.exit(1)
    
    # Run tests
    run_cargo_test()
    
    print("\n" + "=" * 50)
    print("📊 Résumé des tests:")
    print("Les tests Rust peuvent être exécutés même si certaines")
    print("fonctionnalités de haut niveau ne sont pas encore implémentées.")
    print("Cela permet de valider la base solide du code.")

if __name__ == "__main__":
    main()