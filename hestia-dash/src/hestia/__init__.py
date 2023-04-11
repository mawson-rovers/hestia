from .board import Hestia as Hestia
from .stub_board import StubHestia as StubHestia
from .heater import HeaterMode as HeaterMode

stub_instance = StubHestia()  # singleton instance for Flask app
