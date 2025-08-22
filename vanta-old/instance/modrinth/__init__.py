import datetime
import json
import shutil
import sys
from pathlib import Path
from typing import TypedDict

import minecraft_launcher_lib as mll
import requests
import typer
from minecraft_launcher_lib.types import MinecraftVersionInfo

from vanta.instance.util import load_instance_info, vanta_instance_dir

modrinth_group = typer.Typer()
API = "https://api.modrinth.com/v2"
UA = "spelis/vanta/0.1 (spelis.loves.rust@gmail.com)"  # Required by Modrinth API


class VersionInfo(TypedDict, total=False):
    id: str
    releaseTime: str


def filter_and_sort_versions(
    versions: list[dict[str, list[str]]],
    mc_version: str,
    allowed_loaders: list[str],
) -> list[dict[str, list[str]]]:
    """
    Filter versions by Minecraft version and allowed loaders,
    then sort them by release date (descending).
    """
    # Step 1: Filter
    filtered = [
        v
        for v in versions
        if mc_version in v.get("game_versions", [])
        and any(loader in allowed_loaders for loader in v.get("loaders", []))
    ]
    # Step 2: Sort by release date
    return sort_versions(filtered, mc_version, allowed_loaders)


def sort_versions(
    versions: list[dict[str, list[str]]],
    mc_version: str,
    allowed_loaders: list[str],
) -> list[dict[str, list[str]]]:
    """Sort versions by release time (newest first)."""
    all_versions: list[MinecraftVersionInfo] = mll.utils.get_version_list()

    release_lookup = {
        v["id"]: datetime.datetime.fromisoformat(v["releaseTime"])
        for v in all_versions
        if "id" in v and "releaseTime" in v
    }

    sorted_versions = sorted(
        versions,
        key=lambda v: (
            release_lookup.get(v.get("game_versions", [None])[-1])
            if v.get("game_versions")
            else None
        ),
        reverse=True,
    )
    return sorted_versions


def http(endpoint: str, **params):
    resp = requests.get(API + endpoint, params=params, headers={"User-Agent": UA})
    resp.raise_for_status()
    return resp.json()


def format_hits(hits):
    if not hits:
        print("No results.")
        return

    # Determine column widths
    max_slug = max((len(p["slug"]) for p in hits), default=4)
    max_type = max((len(p["project_type"]) for p in hits), default=4)
    max_dl = max((len(str(p["downloads"])) for p in hits), default=2)

    fmt = f"{{slug:<{max_slug}}}  {{type:<{max_type}}}  {{downloads:>{max_dl}}}  {{title}}"
    print(fmt.format(slug="SLUG", type="TYPE", downloads="DL", title="TITLE"))
    print("-" * (max_slug + max_type + max_dl + 10))

    for p in hits:
        print(
            fmt.format(
                slug=p["slug"],
                type=p["project_type"],
                downloads=p["downloads"],
                title=p["title"],
            )
        )


@modrinth_group.command()
def search(
    ctx: typer.Context,
    query: str = typer.Argument(...),
    project_type: str | None = typer.Option(
        None, "--type", help="mod, modpack, resourcepack, shader"
    ),
    limit: int = typer.Option(20, "--limit"),
):
    """
    Search Modrinth for projects.
    """
    instance = ctx.obj.instance
    inst_info = load_instance_info(instance)
    version = inst_info["version"]
    loader = inst_info["loaderName"]

    facets = []
    if project_type:
        facets.append(f"project_type:{project_type}")
    if loader:
        facets.append(f"categories:{loader}")
    if version:
        facets.append(f"versions:{version}")

    params = {"query": query, "limit": limit}
    if facets:
        # each facet must be its own list per API spec
        params["facets"] = json.dumps([[f] for f in facets])

    data = http("/search", **params)
    hits = data.get("hits", [])
    format_hits(hits)


def print_progress(downloaded, total, status="Downloading"):
    term_width = shutil.get_terminal_size((80, 20)).columns
    percent = (downloaded / total * 100) if total else 0
    bar_length = 15
    filled = int(bar_length * percent / 100) if total else 0
    bar = "=" * filled + "-" * (bar_length - filled)
    line = f"[{bar}] {percent:6.2f}% - {status}"
    if len(line) > term_width - 1:
        line = line[: term_width - 4] + "..."
    print(f"\r\x1b[2K{line}", end="", flush=True)


def download_with_progress(url, filename):
    with requests.get(url, stream=True) as r:
        r.raise_for_status()
        total = int(r.headers.get("content-length", 0))
        downloaded = 0
        with open(filename, "wb") as f:
            for chunk in r.iter_content(chunk_size=8192):
                if chunk:
                    f.write(chunk)
                    downloaded += len(chunk)
                    print_progress(downloaded, total)
    print()  # newline when done


class ProjectData(TypedDict):
    project_type: str
    versions: list[dict[str, list[str] | dict[str, str] | str]]


_PROJECT_DIRS: dict[str, str] = {
    "mod": "mods",
    "resourcepack": "resourcepacks",
    "shader": "shaders",
}


@modrinth_group.command("download")
def download(
    ctx: typer.Context,
    slug: str,
    *,
    override_game_version: str | None = typer.Option(
        None, help="Force a Minecraft version to match"
    ),
    override_loader: str | None = typer.Option(
        None, help="Force a loader (e.g. 'fabric', 'forge')"
    ),
) -> None:
    """
    Download the latest Modrinth project version matching your instance.
    """
    obj = ctx.obj
    inst_info: dict[str, str] = load_instance_info(obj.instance)

    game_version = override_game_version or inst_info["version"]
    loader_name = override_loader or inst_info["loaderName"]

    print(f'Identifying Modrinth project "{slug}"...')

    proj_data: ProjectData = http(f"/project/{slug}")
    proj_type = proj_data["project_type"]
    print(f"Detected project type: {proj_type}")

    if proj_type not in _PROJECT_DIRS:
        print(f"Error: Project type '{proj_type}' not supported", file=sys.stderr)
        sys.exit(1)

    save_subdir = _PROJECT_DIRS[proj_type]
    print("Retrieving version list...", end=" ")

    # fetch with game version filter
    ver_list: list[dict[str, list[str] | dict[str, str] | str]] = http(
        f"/project/{slug}/version",
        **{"game_versions": [game_version]},
    )

    if proj_type == "mod":
        ver_list: list[dict[str, list[str]]] = filter_and_sort_versions(
            ver_list, mc_version=game_version, allowed_loaders=[loader_name]
        )
    else:
        ver_list = sort_versions(
            ver_list, mc_version=game_version, allowed_loaders=[loader_name]
        )

    if not ver_list:
        print(
            f"No matching versions found for loader='{loader_name}' game='{game_version}'",
            file=sys.stderr,
        )
        sys.exit(0)

    chosen = ver_list[0]
    print(f"\nUsing version: {chosen['name']}")

    inst_dir = Path(obj.base_dir) / obj.instance_name / ".minecraft" / save_subdir
    inst_dir.mkdir(parents=True, exist_ok=True)

    file_info = chosen["files"][0]
    dest = inst_dir / file_info["filename"]

    print(f"Downloading to {dest}...")
    download_with_progress(file_info["url"], dest)

    print("Done!")
