@echo off
REM Script de compilation et d'installation de l'adaptateur MT5 pour Windows
REM Ce script crée l'adaptateur MT5 fonctionnel avec les bindings Python

echo 🔧 Construction de l'adaptateur MT5 - NautilusTrader (Windows)
echo ============================================================

REM Variables
set PROJECT_ROOT=.
set MT5_ADAPTER_DIR=%PROJECT_ROOT%\crates\adapters\mt5

REM Vérifier la présence de Rust
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Rust/Cargo n'est pas installé. Veuillez installer Rust :
    echo    https://rustup.rs
    pause
    exit /b 1
)

REM Vérifier la présence de Python
where python >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Python n'est pas installé. Veuillez installer Python 3.8+
    pause
    exit /b 1
)

echo ✅ Rust et Python détectés

REM Étape 1: Nettoyer les builds précédents
echo.
echo 🧹 Nettoyage des builds précédents...
cd /d "%MT5_ADAPTER_DIR%"
if exist target rmdir /s /q target
if exist dist rmdir /s /q dist

REM Étape 2: Vérifier les dépendances Rust
echo.
echo 📦 Vérification des dépendances Rust...
if not exist Cargo.toml (
    echo ❌ Cargo.toml non trouvé dans %MT5_ADAPTER_DIR%
    pause
    exit /b 1
)

REM Étape 3: Compiler l'adaptateur (debug)
echo.
echo 🔨 Compilation de l'adaptateur MT5 (debug)...
cargo build
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Échec de la compilation debug
    pause
    exit /b 1
)
echo ✅ Compilation debug réussie

REM Étape 4: Compiler avec bindings Python
echo.
echo 🐍 Compilation avec bindings Python...
cargo build --release --features python-bindings
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Échec de la compilation avec Python bindings
    pause
    exit /b 1
)
echo ✅ Compilation avec Python bindings réussie

REM Étape 5: Installer les outils Python nécessaires
echo.
echo 📦 Installation des outils Python...
python -m pip install --user maturin build wheel
if %ERRORLEVEL% NEQ 0 (
    echo ⚠️  Avertissement: Échec de l'installation de maturin
)

REM Étape 6: Créer le package Python
echo.
echo 📦 Création du package Python...
python -m maturin build --release --features python-bindings --out target\wheels
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Échec de la création du package Python
    pause
    exit /b 1
)
echo ✅ Package Python créé dans target\wheels\

REM Étape 7: Installer le package
echo.
echo 📥 Installation du package Python...
for /f "delims=" %%i in ('dir /b target\wheels\*.whl 2^>nul') do (
    set WHEEL_FILE=target\wheels\%%i
    goto :install
)
echo ❌ Aucun fichier .whl trouvé
pause
exit /b 1

:install
python -m pip install --user "%WHEEL_FILE%"
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Échec de l'installation du package
    pause
    exit /b 1
)
echo ✅ Package installé: %WHEEL_FILE%

REM Étape 8: Test d'import
echo.
echo 🧪 Test d'import de l'adaptateur...
python -c "import sys; sys.path.append('.'); from nautilus_trader.adapters.mt5 import Mt5Config; print('✅ Import Mt5Config: OK')"
python -c "import sys; sys.path.append('.'); from nautilus_trader.adapters.mt5 import Mt5HttpClient; print('✅ Import Mt5HttpClient: OK')"
python -c "import sys; sys.path.append('.'); from nautilus_trader.adapters.mt5 import Mt5WebSocketClient; print('✅ Import Mt5WebSocketClient: OK')"

echo.
echo 🎉 Construction terminée !
echo.
echo 📋 Résumé:
echo    - Code Rust: ✅ Compilé
echo    - Bindings Python: ✅ Générés
echo    - Package: ✅ Installé
echo.
echo 💡 Prochaines étapes:
echo    1. Installer NautilusTrader: pip install nautilus-trader
echo    2. Tester l'adaptateur: python -c "from nautilus_trader.adapters.mt5 import *; print('✅ Adaptateur disponible')"
echo    3. Configurer un serveur MT5 bridge
echo.
echo 📖 Consultez README_MT5_INTEGRATION.md pour l'utilisation

pause