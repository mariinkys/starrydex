<p align="center">
 <img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/res/icons/hicolor/256x256/apps/dev.mariinkys.StarryDex.svg">
</p>

<h1 align="center">StarryDex</h1>

<p align="center">
 This project contains a small Pokédex application for the COSMIC™ desktop written in Rust with <a href="https://github.com/pop-os/libcosmic" target="_blank">libcosmic</a>.
</p>

<p align="center">
 <img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/screenshots/main.png" width=350>
 <img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/screenshots/pokemon.png" width=350>
</p>

## Information

> [!WARNING]
> This application is being developed for learning purposes and it may or may not be usable.
> Due to time constrains, development may be slow for a while.

Created by [mariinkys](https://github.com/mariinkys). Pokémon and Pokémon character names are trademarks of Nintendo.

This application uses [PokeApi](https://github.com/PokeAPI/) and it's resources. 

## Install

To install your COSMIC application, you will need [just](https://github.com/casey/just), if you're on Pop!\_OS, you can install it with the following command:

```sh
sudo apt install just
```

After you install it, you can run the following commands to build and install your application:

```sh
just build-release
sudo just install
```
