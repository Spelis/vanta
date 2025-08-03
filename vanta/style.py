def ask(prompt: str) -> str:
    val: str = input(f"\x1b[1;35m{prompt}: \x1b[0;37m")
    print("\x1b[0m", end="")
    return val
