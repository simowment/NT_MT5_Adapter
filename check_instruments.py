try:
    import nautilus_trader.model.instruments as instruments

    print([name for name in dir(instruments) if name[0].isupper()])
except ImportError as e:
    print(e)
