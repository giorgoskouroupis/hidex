# tIDEx: 'python -m pudb' and breakpoint() in any env, without installing pudb there. Its pure-Python copy
# (lib/, next to this file) goes at the end of the import path, so the env's own packages always come first.
import os
import sys

_here = os.path.dirname(os.path.abspath(__file__))
_lib = os.path.join(_here, "lib")
if os.path.isdir(_lib) and _lib not in sys.path:
    sys.path.append(_lib)


# pudb's settings module is given its folders as it is imported (the program's environment is untouched):
# its state goes to ~/.local/state/tidex/pudb, never ~/.config, and tIDEx's settings, read last, win.
class _TidexPudbSettings:
    def find_spec(self, name, path=None, target=None):
        if name != "pudb.settings":
            return None
        sys.meta_path.remove(self)
        import importlib.util

        spec = importlib.util.find_spec(name)
        run = spec.loader.exec_module

        def exec_module(module):
            run(module)
            state = os.path.join(os.environ.get("XDG_STATE_HOME") or os.path.expanduser("~/.local/state"), "tidex")
            xdg = os.path.join(os.path.dirname(_here), "xdg")
            module.XDG_CONFIG_HOME = state
            module.XDG_CONFIG_DIRS = [state, xdg]
            # pudb.cfg names its theme "@theme@": tIDEx's theme.py, wherever tIDEx is installed
            load = module.load_config

            def load_config(*args, **kwargs):
                conf = load(*args, **kwargs)
                if conf.get("theme") == "@theme@":
                    conf["theme"] = os.path.join(xdg, "pudb", "theme.py")
                return conf

            module.load_config = load_config

        spec.loader.exec_module = exec_module
        return spec


sys.meta_path.insert(0, _TidexPudbSettings())
