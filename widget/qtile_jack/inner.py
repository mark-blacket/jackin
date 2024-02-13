import libqtile.widget.base as base
from .jax import *

class _JackWidget(base.ThreadPoolText):
    defaults = [("update_interval", 2, "Update interval in seconds")]

    def __init__(self, conns_file, **cfg):
        client_init()
        super().__init__("", **cfg)

class XrunWidget(_JackWidget):
    def poll(self):
        count, delay, max_delay = xrun_stats()
        return f"{count}, {delay} ({max_delay})"

    def reset(self):
        xrun_reset()
        self.draw()

class CPUWidget(_JackWidget):
    def poll(self):
        return str(cpu_stats())
