# Vanta - The Minecraft launcher for basic and advanced users

Vanta is a lightweight command-line Minecraft launcher written in Rust. It is designed to make managing and playing Minecraft effortless. No clunky GUI *required*.

It handles everything from logging in to installing mods and managing multiple instances. (Python branch has slightly more features than the master (Rust) branch.)

---

## Features

- **Authentication** - Secure login with Microsoft accounts

- **Skins** - Manage and apply player skins easily. (WIP)

- **Launching** - Start Minecraft directly from the Terminal.

- **Mods** - Search, download and automatically install mods, resourcepacks and shaders with ease! (Python Version only)

- **Modloaders** - Automatic installation of Forge, Fabric and Quilt. (Python Version only)

- **Instancing** - Keep multiple modded or vanilla profiles side-by-side

- **More coming soon** - Stay tuned for (Or contribute) new features and integrations!

---

## Installation

```sh
git clone https://github.com/spelis/vanta.git
cd vanta
uv tool install . -e # You will need UV installed!
```

## Usage

First you'll need to log in. To do that you should run:
```sh
vanta user login
```

Then you need an instance to run! Creating a new instance is done as such:
```sh
vanta instance new <instance id> <version>
```

If you aren't sure which version to pick, you can find a list of versions by running:
```sh
vanta instance version-list
# or, if you just want the latest:
vanta instance version-latest
```

Well now, you're getting inpatient! "How do i run the game then?!" I got you. Do this:
```sh
vanta instance <instance id> launch <your username>
```

But lets say you're a little more advanced, you dont play vanilla! You play **modded**. I still got you!
```sh
vanta modloader install <instance id> <modloader>
```
Easy peasy isn't it?

But, you ask: "What's the point of a mod loader without mods?" I **STILL** got you!
```sh
vanta modrinth download <instance id> <mod id>
```

Now, when you launch the game (see previous instruction) you will load into the game with your installed mod loader.

## Roadmap

* [ ] Support for additional modloaders
* [ ] Skin management
* [ ] Custom launch options
* [ ] Rust rewrite, goal is to have the same features as Python version
* [ ] Cracked accounts (might get scrapped)

## Contributing

Contributions are welcome! (I'm begging of you to contribute)

If you'd like to help, feel free to open an issue, submit a PR, or suggest new features.

Make sure you read the [contribution](CONTRIBUTING.md) file if you wanna write code.

## License

This project is licensed under the **MIT License**.

It is a permissive license used in lots of Free and Open Source software.

See the [LICENSE](LICENSE) file for details.
