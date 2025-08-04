import json
import os
import urllib
import uuid
import webbrowser
from pathlib import Path

import minecraft_launcher_lib as mll
import platformdirs
import typer
from dotenv import load_dotenv

vanta_data_path = platformdirs.user_data_path("vanta", "spelis")
if not vanta_data_path.exists():
    print("Vanta directory not initialized... Creating...")
    Path.mkdir(vanta_data_path)

user_group = typer.Typer(help="User Control")

load_dotenv()

MSA_CLIENT_ID = os.environ.get("MSA_CLIENT_ID", None)
MSA_REDIRECT_URL = os.environ.get("MSA_REDIRECT_URL", None)
SCOPE = ["XboxLive.signin", "offline_access"]

# Where we store user data
DATA_FILE = vanta_data_path / ".accounts.json"


def prompt_auth():
    state = uuid.uuid4().hex
    auth_url, state, verifier_obj = mll.microsoft_account.get_secure_login_data(
        MSA_CLIENT_ID, MSA_REDIRECT_URL
    )

    webbrowser.open_new_tab(auth_url)
    auth_query = urllib.parse.parse_qs(
        input("Paste query string from URL (code=...&state=...): ").strip()
    )

    if auth_query.get("state", [""])[0] != state:
        raise ValueError("state mismatch")

    code = auth_query["code"][0]

    info = mll.microsoft_account.complete_login(
        MSA_CLIENT_ID, None, MSA_REDIRECT_URL, code, verifier_obj
    )
    return info


def load_accounts():
    if DATA_FILE.exists():
        return json.loads(DATA_FILE.read_text())
    return {}


def save_accounts(accounts):
    DATA_FILE.write_text(json.dumps(accounts, indent=2))


def login(cracked_username: str = "") -> bool:
    """[LIB] Log in a user."""
    if not cracked_username:
        entry = prompt_auth()
    else:
        print(
            """\x1b[31m⚠️ Important Notice ⚠️
This launcher allows offline (cracked) play.
Please be aware that using cracked accounts may be illegal
in your country and is not supported by Mojang or Microsoft.  
We recommend purchasing Minecraft at https://www.minecraft.net 
to support the developers and ensure full access to the game.\x1b[0m"""
        )
        entry = {
            "id": "00000000-0000-0000-0000-000000000000",
            "name": cracked_username,
            "cracked": True,
        }
    accounts = load_accounts()
    accounts[entry["name"]] = entry
    save_accounts(accounts)
    print(f"✅ Logged in as {entry['name']} ({entry['id']})")
    return True


def logout(user: str) -> bool:
    """[LIB] Log out user"""
    accounts = load_accounts()
    if user not in accounts:
        print(f"No user named {user} found")
        return False
    del accounts[user]
    save_accounts(accounts)
    return True


def list() -> dict[str, str]:
    """[LIB] List users"""
    accounts = load_accounts()
    return {name: data["id"] for name, data in accounts.items()}


@user_group.command("login")
def _user_login(crack_username: str = ""):
    """Log-in a user"""
    if login(crack_username):
        print("Success!")


@user_group.command("logout")
def _user_logout(username: str):
    """Log-out a specific user"""
    if logout(username):
        print("Success!")


@user_group.command("list")
def _user_list():
    """List all logged-in users"""
    users = list()
    if not users:
        print("No users logged in")
    else:
        for name, uuid in users.items():
            print(f"{name} ({uuid})")
