<div align="center">
<br>
<img src="./resources/icons/hicolor/scalable/apps/icon.svg" width="150" />
<h1 align="center">StarryDex</h1>

![Flathub Version](https://img.shields.io/flathub/v/dev.mariinkys.StarryDex)
![Flathub Downloads](https://img.shields.io/flathub/downloads/dev.mariinkys.StarryDex)
![GitHub License](https://img.shields.io/github/license/mariinkys/starrydex)
![GitHub Repo stars](https://img.shields.io/github/stars/mariinkys/starrydex)

<h3>A Pokédex application for the COSMIC™ desktop</h3>

<img src="./resources/screenshots/main-dark.png" width=350>
<img src="./resources/screenshots/pokemon-dark.png" width=350>

<br><br>

<a href="https://flathub.org/apps/dev.mariinkys.StarryDex">
   <img width='240' alt='Download on Flathub' src='https://flathub.org/api/badge?locale=en'/>
 </a>
 </div>

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

## Attribution

> "[Pop Icons](http://github.com/pop-os/icon-theme)" by [System76](http://system76.com/) is licensed under [CC-SA-4.0](http://creativecommons.org/licenses/by-sa/4.0/)

> "Pokémon Type Icons" originally made by [duiker101](https://github.com/duiker101/pokemon-type-svg-icons)

Pokémon and Pokémon character names are trademarks of Nintendo.

This application uses [PokeApi](https://github.com/PokeAPI/) and its resources.

## Development

To update/generate your assets you have to execute: `cargo run -p assetgen -- -a`
```
-a = All Assets
-p = Only Pokémon Data
-s = Only Sprites
```

## Copyright and Licensing

Copyright 2024 © Alex Marín

Released under the terms of the [GPL-3.0](./LICENSE)