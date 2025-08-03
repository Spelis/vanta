import datetime
import os
import shutil
import subprocess
import sys
from pathlib import Path
from types import SimpleNamespace

import minecraft_launcher_lib as mll
import requests
import typer
from minecraft_launcher_lib.types import MinecraftOptions

from vanta.instance import loaders, modrinth, util
from vanta.mll_callbacks import make_callbacks
from vanta.user import load_accounts, vanta_data_path

instance_group = typer.Typer(help="Instance Control")

instance_group.add_typer(modrinth.modrinth_group, name="modrinth")
instance_group.add_typer(loaders.loader_group, name="loader")


def get_all_versions():
    resp = requests.get(
        "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json", timeout=15
    )
    resp.raise_for_status()
    return resp.json()["versions"]


def new(name: str, id: str, version: str) -> bool:
    """[LIB] Create a new Instance"""
    mll.install.install_minecraft_version(
        version, str(util.vanta_instance_dir / id / ".minecraft"), make_callbacks()
    )
    print(f"\r\x1b[2KInstalled {version}!\n", end="", flush=True)
    info = {
        "name": name,
        "version": version,
        "jarVersion": version,
        "loaderName": "vanilla",
        "group": "Ungrouped",
    }
    util.save_instance_info(id, info)
    return True


def delete(identifier: str):
    shutil.rmtree(str(util.vanta_instance_dir / identifier))


def launch(
    identifier: str,
    user: str,
    instance_dir: Path,
    accounts: dict[str, dict[str, str]],
) -> None:
    """
    [LIB] Launches the specified instance
    Args:
        identifier (str): the instance ID to launch
        user (str): the user to launch minecraft as
        instance_dir (Path): the path to the instance directory
        accounts: (dict): all logged in users, see accounts.json
    """
    if not user:
        raise ValueError("No user specified. Please pass --user")
    inst = instance_dir / identifier
    mcdir = inst / ".minecraft"
    if not mcdir.exists():
        raise ValueError(f"Instance or .minecraft not found for '{identifier}'")

    inst_info = util.load_instance_info(identifier)  # you already have
    version = inst_info["jarVersion"]

    login_data = accounts.get(user)
    if login_data is None:
        raise ValueError(f"User '{user}' not logged in")

    opts: MinecraftOptions = {
        "username": login_data["name"],
        "uuid": login_data["id"],
        "token": login_data["access_token"],
    }
    cmd = mll.command.get_minecraft_command(version, mcdir, opts)
    print("The game is launching, only Warnings and Errors will be printed.")
    _ = subprocess.run(cmd, cwd=mcdir, check=True, stdout=subprocess.DEVNULL)


@instance_group.callback(invoke_without_command=False)
def _instance_alias(
    ctx: typer.Context, instance: str = typer.Argument(..., help="Instance identifier")
):
    """
    Instance Alias
    """
    ctx.obj = SimpleNamespace(instance=instance)


@instance_group.command("new")
def _instance_new(ctx: typer.Context, version: str, name: str = ""):
    """Create a new Instance"""
    identifier = ctx.obj.instance
    if not name:
        name = identifier.capitalize()
    if new(name, identifier, version):
        print("Success")


@instance_group.command("launch")
def _instance_launch(
    ctx: typer.Context,
    user: str = typer.Argument(..., help="Username from who to log in"),
):
    """Launch an Instance"""
    identifier = ctx.obj.instance
    launch(identifier, user, util.vanta_instance_dir, load_accounts())
    typer.echo("✅ Game exited cleanly")


@instance_group.command("list")
def _instance_list(
    ctx: typer.Context,
):
    instances = os.listdir(str(util.vanta_instance_dir))
    header = f"{'ID':20} {'Loader':8} {'Version':11} {'Group':20}"
    print(header)
    print("-" * len(header))
    nv = 0
    for i in instances:  # i fucking hate myself for this, could've been a map() but nah
        i_info = util.load_instance_info(i)
        print(
            f"{i:20} {i_info['loaderName']:8} {i_info['version']:11} {i_info['group']:20}"
        )


@instance_group.command("delete")
def _instance_delete(ctx: typer.Context):
    instance = ctx.obj.instance
    delete(instance)


@instance_group.command("version_list")
def _instance_list_versions(
    precision: str = "release",
):
    versions = get_all_versions()
    header = f"{'ID':20} {'Type':8} {'ReleaseTime'}"
    print(header)
    print("-" * len(header))
    nv = 0
    for v in versions:  # i fucking hate myself for this, could've been a map() but nah
        if precision != "all":
            if v["type"] != precision:
                continue
        nv += 1
        print(
            f"{v['id']:20} {v['type']:8} {datetime.datetime.fromisoformat(v['releaseTime'].replace('Z','+00:00')).strftime("%D")}"
        )
    print(f"listed {nv}/{len(versions)} versions")


@instance_group.command("version_latest")
def _instance_latest_version(precision: str = "release"):
    print(mll.utils.get_latest_version()[precision])
