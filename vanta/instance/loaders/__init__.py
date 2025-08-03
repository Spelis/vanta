from pathlib import Path
from types import SimpleNamespace

import minecraft_launcher_lib as mll
import typer

from vanta.instance.util import (
    load_instance_info,
    save_instance_info,
    vanta_instance_dir,
)
from vanta.mll_callbacks import make_callbacks

loader_group = typer.Typer(help="Mod loader management")


@loader_group.callback(invoke_without_command=False)
def _loader_alias(
    ctx: typer.Context,
    loader: str = typer.Argument(..., help="The mod loader to manage"),
):
    """Loader manager"""
    ctx.obj.loader = loader


@loader_group.command("install")
def _loader_install(
    ctx: typer.Context,
):
    loader: str = ctx.obj.loader
    instance: str = ctx.obj.instance
    inst_info: dict[str, str] = load_instance_info(instance)
    inst_dir: Path = vanta_instance_dir / instance / ".minecraft"
    callbacks = make_callbacks()
    if loader == "fabric":
        version: str = str(mll.fabric.get_latest_loader_version())
        mll.fabric.install_fabric(inst_info["version"], inst_dir, version, callbacks)
        inst_info["jarVersion"] = f"fabric-loader-{version}-{inst_info['version']}"
        inst_info["loaderName"] = "fabric"
        save_instance_info(instance, inst_info)
    if loader == "forge":
        version: str = str(mll.forge.find_forge_version(inst_info["version"]))
        mll.forge.install_forge_version(version, inst_dir, callbacks)
        inst_info["jarVersion"] = "forge " + version
        inst_info["loaderName"] = "forge"
        save_instance_info(instance, inst_info)
    if loader == "quilt":
        mll.quilt.install_quilt(inst_info["version"], inst_dir, callbacks)
        print(
            "quilt does not get automatically installed into the instance data (yet), developer is lazy :("
        )
