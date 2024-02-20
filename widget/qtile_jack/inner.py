from libqtile.widget import base
from libqtile.lazy import lazy

from .jax import *

class _JackWidget(base.InLoopPollText):
    defaults = [("update_interval", 2, "Update interval in seconds")]

    def try_init(self):
        try:
            client_init()
            self.initialized = True
        except Exception:
            self.initialized = False
        
    def __init__(self, **cfg):
        self.try_init()
        super().__init__("", **cfg)
        self.add_defaults(_JackWidget.defaults)

    def poll(self, **_):
        if not self.initialized:
            self.try_init()
            return "--"
        else:
            return self.value()

class XrunWidget(_JackWidget):
    def value(self):
        count, delay, max_delay = xrun_stats()
        return f"{count} ({delay:.3}/{max_delay:.3})"

    @lazy.function
    def reset(self):
        xrun_reset()

class CPUWidget(_JackWidget):
    def value(self):
        return f"{cpu_stats():.3}"
