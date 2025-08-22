import os
import tomllib

import platformdirs

from . import instance, user


def validate_config() -> dict[str, int]:
    with open(os.path.join(platformdirs.user_config_dir(), "vanta.toml"), "w+") as f:
        return tomllib.loads(f.read())
