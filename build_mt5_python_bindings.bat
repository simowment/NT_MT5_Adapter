@echo off
REM Script pour construire les bindings Python MT5 pour NautilusTrader
REM 
REM Prérequis:
REM - Rust et cargo installés
REM - Python 3.8+ installé
REM - maturin installé: pip install maturin
REM - VS Build Tools (pour Windows)

echo === Construction des bindings Python MT5 ===

REM Aller dans le répertoire de l'adaptateur MT5
cd /d "crates\adapters\mt5"

echo Répertoire actuel: %CD%

REM Vérifier que Rust est installé
echo Vérification de Rust...
cargo --version
if %ERRORLEVEL% NEQ 0 (
    echo ERREUR: Rust n'est pas installé ou pas dans le PATH
    pause
    exit /b 1
)

REM Vérifier que maturin est installé
echo Vérification de maturin...
python -c "import maturin" 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Installation de maturin...
    pip install maturin
    if %ERRORLEVEL% NEQ 0 (
        echo ERREUR: Impossible d'installer maturin
        pause
        exit /b 1
    )
)

REM Construire les bindings Python
echo Construction des bindings Python avec maturin...
maturin build --release --features python-bindings

if %ERRORLEVEL% NEQ 0 (
    echo ERREUR: La construction a échoué
    pause
    exit /b 1
)

echo === Construction terminée avec succès ===
echo Les wheels se trouvent dans: target\wheels\

REM Installer le package en mode développement pour test
echo Installation en mode développement pour test...
cd target\wheels
for %%f in (*.whl) do (
    echo Installation de %%f...
    python -m pip install --force-reinstall %%f
)

cd ..\..\..\..
echo Prêt pour les tests!

pause