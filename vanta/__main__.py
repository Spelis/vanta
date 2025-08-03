import os
import platform
import subprocess

import platformdirs
import typer

from vanta import validate_config
from vanta.instance import instance_group
from vanta.user import user_group

app = typer.Typer(help="Vanta is a CLI Minecraft Launcher")

app.add_typer(user_group, name="user")
app.add_typer(instance_group, name="instance")


@app.command("config")
def open_config():
    """Open vanta.toml Config File"""
    file_path = os.path.join(platformdirs.user_config_dir(), "vanta.toml")
    system = platform.system()
    if system == "Darwin":  # macOS
        subprocess.run(["open", file_path])
    elif system == "Windows":
        os.startfile(file_path)
    else:  # Assume Linux
        subprocess.run(["xdg-open", file_path])


if __name__ == "__main__":  # this is stupid cause this is __main__.py
    _ = validate_config()
    app()
