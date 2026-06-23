"""Package entrypoint so `python3 -m tools.confevo` runs the CLI.

Equivalent to `python3 tools/confevo/confevo.py`.
"""

from __future__ import annotations

import sys

try:  # package invocation: python3 -m tools.confevo
    from .confevo import main
except ImportError:  # flat fallback: directory on sys.path
    from confevo import main  # type: ignore[no-redef]

if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
