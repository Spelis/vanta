from types import FunctionType


def make_callbacks() -> dict[str, FunctionType]:
    progress: dict[str, float | str] = {"current": 0.0, "max": 1.0, "status": ""}

    def set_status(x: str):
        progress["status"] = x

    def set_max(x: float):
        progress["max"] = x

    def set_progress(x: float):
        progress["current"] = x
        bar_len = 15
        filled_len = int(bar_len * x // float(progress["max"]))
        bar = "=" * filled_len + "-" * (bar_len - filled_len)
        percent = (x / float(progress["max"])) * 100
        print(
            f"\r\x1b[2K" + f"[{bar}] {percent:.1f}% - {progress['status']}",
            end="",
            flush=True,
        )

    return {
        "setStatus": set_status,
        "setProgress": set_progress,
        "setMax": set_max,
    }
