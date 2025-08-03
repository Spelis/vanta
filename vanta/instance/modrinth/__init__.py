import json
import shutil
from typing import Optional

import requests
import typer
from packaging.version import Version

from vanta.instance.util import load_instance_info, vanta_instance_dir

modrinth_group = typer.Typer()
API = "https://api.modrinth.com/v2"
UA = "spelis/vanta/0.1 (spelis.loves.rust@gmail.com)"  # Required by Modrinth API


def filter_and_sort_versions(versions, mc_version, allowed_loaders):
    # Step 1: Filter
    filtered = [
        v
        for v in versions
        if mc_version in v.get("game_versions", [])
        and any(loader in allowed_loaders for loader in v.get("loaders", []))
    ]

    # Step 2: Sort descending by version_number
    sorted_versions = sorted(
        filtered, key=lambda v: Version(v["game_versions"][-1]), reverse=True
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
    max_slug, max_type, max_dl = 0, 0, 0
    for p in hits:
        max_slug = max(max_slug, len(p["slug"]))
        max_type = max(max_type, len(p["project_type"]))
        max_dl = max(max_dl, len(str(p["downloads"])))
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
    project_type: Optional[str] = typer.Option(
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
        facets.append(f"categories:{loader}")  # "categories" includes loaders
    if version:
        facets.append(f"versions:{version}")

    params = {"query": query, "limit": limit}
    if facets:
        params["facets"] = json.dumps([[f] for f in facets])

    data = http("/search", **params)
    hits = data.get("hits", [])
    format_hits(hits)


def print_progress(downloaded, total, status="Downloading"):
    term_width = shutil.get_terminal_size((80, 20)).columns
    if total:
        percent = downloaded / total * 100
    else:
        percent = 0  # unknown length

    bar_length = 15  # leave space for % and status
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
    print()  # move to next line when done


@modrinth_group.command()
def download(ctx: typer.Context, slug: str):
    """
    Download a Modrinth project based on its slug.
    """
    instance = ctx.obj.instance
    inst_info = load_instance_info(instance)
    version = inst_info["version"]
    loader = inst_info["loaderName"]
    print("Finding appropriate version...", end="", flush=True)
    data = http(
        f"/project/{slug}/version",
        **{
            "loaders": [
                loader,
            ],
            "game_versions": [
                version,
            ],
        },
    )
    data = filter_and_sort_versions(data, mc_version=version, allowed_loaders=[loader])
    print("\r\x1b[2KFound an appropriate version:", data[0]["name"])
    download_with_progress(
        data[0]["files"][0]["url"],
        vanta_instance_dir
        / instance
        / ".minecraft"
        / "mods"
        / data[0]["files"][0]["filename"],
    )
