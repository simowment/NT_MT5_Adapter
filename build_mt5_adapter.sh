#!/bin/bash
# Script de compilation et d'installation de l'adaptateur MT5

echo "🔧 Construction de l'adaptateur MT5 - NautilusTrader"
echo "===================================================="

# Variables
PROJECT_ROOT="."
MT5_ADAPTER_DIR="$PROJECT_ROOT/crates/adapters/mt5"
VENV_NAME="nautilus_env"

# Vérifier la présence de Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo n'est pas installé. Veuillez installer Rust :"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Vérifier la présence de Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python3 n'est pas installé. Veuillez installer Python 3.8+"
    exit 1
fi

echo "✅ Rust et Python détectés"

# Étape 1: Nettoyer les builds précédents
echo ""
echo "🧹 Nettoyage des builds précédents..."
cd "$MT5_ADAPTER_DIR"
cargo clean
rm -rf target/wheels 2>/dev/null || true
rm -rf dist 2>/dev/null || true

# Étape 2: Vérifier les dépendances Rust
echo ""
echo "📦 Vérification des dépendances Rust..."
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Cargo.toml non trouvé dans $MT5_ADAPTER_DIR"
    exit 1
fi

# Étape 3: Compiler l'adaptateur
echo ""
echo "🔨 Compilation de l'adaptateur MT5 (debug)..."
cargo build

if [ $? -ne 0 ]; then
    echo "❌ Échec de la compilation debug"
    exit 1
fi

echo "✅ Compilation debug réussie"

# Étape 4: Compiler avec bindings Python
echo ""
echo "🐍 Compilation avec bindings Python..."
cargo build --release --features python-bindings

if [ $? -ne 0 ]; then
    echo "❌ Échec de la compilation avec Python bindings"
    exit 1
fi

echo "✅ Compilation avec Python bindings réussie"

# Étape 5: Installer les outils Python nécessaires
echo ""
echo "📦 Installation des outils Python..."
pip3 install --user maturin build wheel

# Étape 6: Créer le package Python
echo ""
echo "📦 Création du package Python..."
maturin build --release --features python-bindings --out target/wheels

if [ $? -ne 0 ]; then
    echo "❌ Échec de la création du package Python"
    exit 1
fi

echo "✅ Package Python créé dans target/wheels/"

# Étape 7: Installer le package
echo ""
echo "📥 Installation du package Python..."
WHEEL_FILE=$(find target/wheels -name "*.whl" | head -1)
if [ -n "$WHEEL_FILE" ]; then
    pip3 install --user "$WHEEL_FILE"
    echo "✅ Package installé: $WHEEL_FILE"
else
    echo "❌ Aucun fichier .whl trouvé"
    exit 1
fi

# Étape 8: Test d'import
echo ""
echo "🧪 Test d'import de l'adaptateur..."
python3 -c "
try:
    from nautilus_trader.adapters.mt5 import Mt5Config
    print('✅ Import Mt5Config: OK')
except ImportError as e:
    print(f'❌ Import Mt5Config: {e}')

try:
    from nautilus_trader.adapters.mt5 import Mt5HttpClient
    print('✅ Import Mt5HttpClient: OK')
except ImportError as e:
    print(f'❌ Import Mt5HttpClient: {e}')

try:
    from nautilus_trader.adapters.mt5 import Mt5WebSocketClient  
    print('✅ Import Mt5WebSocketClient: OK')
except ImportError as e:
    print(f'❌ Import Mt5WebSocketClient: {e}')
"

echo ""
echo "🎉 Construction terminée !"
echo ""
echo "📋 Résumé:"
echo "   - Code Rust: ✅ Compilé"
echo "   - Bindings Python: ✅ Générés"
echo "   - Package: ✅ Installé"
echo ""
echo "💡 Prochaines étapes:"
echo "   1. Installer NautilusTrader: pip3 install nautilus-trader"
echo "   2. Tester l'adaptateur: python3 -c 'from nautilus_trader.adapters.mt5 import *; print(\"✅ Adaptateur disponible\")'"
echo "   3. Configurer un serveur MT5 bridge"
echo ""
echo "📖 Consultez README_MT5_INTEGRATION.md pour l'utilisation"