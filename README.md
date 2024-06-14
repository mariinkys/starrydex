<p align="center">
 <img src="https://raw.githubusercontent.com/mariinkys/starrydex/5546d9d50c7d99a3cb3fc882a955a8eb274d0949/res/icons/hicolor/256x256/apps/dev.mariinkys.StarryDex.svg?token=AO2BUSPXV272EVQCF7C4ILTGLWFIU">
</p>

<h1 align="center">StarryDex</h1>

<p align="center">
 This project contains a small Pokédex application for the COSMIC™ desktop written in Rust with <a href="https://github.com/pop-os/libcosmic" target="_blank">libcosmic</a>.
</p>

<p align="center">
 <img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/screenshots/main.png" width=450>
 <img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/screenshots/pokemon.png" width=450>
</p>

## Information

<b>Due to health issues and time constrains, development may be slow for a while.</b>

<b>This application is being developed for learning purposes and it may or may not be usable.</b>

Created by [mariinkys](https://github.com/mariinkys). Pokémon and Pokémon character names are trademarks of Nintendo.

This application uses [PokeApi](https://github.com/PokeAPI/) and it's resources. [PokeApi](https://github.com/PokeAPI/) resources comply with [PokeApi](https://github.com/PokeAPI/)'s [LICENSE](https://github.com/mariinkys/starrydex/blob/main/resources/LICENSE.md)

## Install

*Note that for now, the resources folder that should be installed alongside the application is not being installed, which means that the application may not function correctly.

To install your COSMIC application, you will need [just](https://github.com/casey/just), if you're on Pop!\_OS, you can install it with the following command:

```sh
sudo apt install just
```

After you install it, you can run the following commands to build and install your application:

```sh
just build-release
sudo just install
```
