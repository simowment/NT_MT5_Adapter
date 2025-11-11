from flask import Flask, request, jsonify, redirect
import MetaTrader5 as mt5
import inspect

app = Flask(__name__)

# Initialisation MT5
if not mt5.initialize():
    raise RuntimeError("MetaTrader5 initialization failed")

def call_mt5_function(func_name, params):
    func = getattr(mt5, func_name, None)
    if not func or not callable(func):
        return {"error": f"Function '{func_name}' not found"}

    try:
        # Exécution directe
        result = func(**params) if params else func()
        if hasattr(result, "_asdict"):
            result = result._asdict()
        elif isinstance(result, tuple):
            result = [r._asdict() if hasattr(r, "_asdict") else r for r in result]
        return {"result": result}
    except TypeError as e:
        return {"error": f"Bad arguments: {e}"}
    except Exception as e:
        return {"error": str(e)}

# Génération dynamique des endpoints
for name, func in inspect.getmembers(mt5, inspect.isbuiltin):
    if name.startswith("__"):
        continue

    def make_endpoint(f_name):
        def endpoint():
            params = request.get_json(force=True, silent=True) or {}
            return jsonify(call_mt5_function(f_name, params))
        endpoint.__name__ = f_name
        app.route(f"/api/{f_name}", methods=["POST"])(endpoint)

    make_endpoint(name)

@app.route("/", methods=["GET"])
def root():
    # Redirection vers /api/docs pour offrir une interface lisible
    return redirect("/api/docs", code=302)

@app.route("/api", methods=["GET"])
def list_endpoints():
    return jsonify({
        "available_endpoints": [
            rule.rule for rule in app.url_map.iter_rules()
            if rule.rule.startswith("/api/") and rule.rule != "/api" and rule.rule != "/api/docs"
        ]
    })

@app.route("/api/docs", methods=["GET"])
def docs():
    """
    Retourne une documentation simple des endpoints dynamiques MT5 disponibles.
    Clé: nom de la fonction MetaTrader5
    Valeur: endpoint HTTP correspondant
    """
    funcs = {}
    for name, func in inspect.getmembers(mt5, inspect.isbuiltin):
        if name.startswith("__"):
            continue
        funcs[name] = {
            "endpoint": f"/api/{name}",
            "method": "POST",
            "description": "Proxy HTTP vers mt5.%s(**json_body)" % name
        }
    return jsonify(funcs)

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000)
