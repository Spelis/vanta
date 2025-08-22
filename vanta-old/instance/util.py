import json
from pathlib import Path

from vanta.user import vanta_data_path

vanta_instance_dir = vanta_data_path / "instances"

if not vanta_instance_dir.exists():
    Path.mkdir(vanta_instance_dir)


def load_instance_info(id: str) -> dict[str, str]:
    j_path = vanta_instance_dir / id / "instance.json"
    if j_path.exists():
        return json.loads(j_path.read_text())
    return {}


def save_instance_info(id: str, data: dict[str, str]):
    j_path = vanta_instance_dir / id / "instance.json"
    j_path.write_text(json.dumps(data))
