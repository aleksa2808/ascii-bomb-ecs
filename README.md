# ascii-bomb-ecs

Port of the [ascii-bomb](https://github.com/aleksa2808/ascii-bomb) game in Rust.

I ditched the `pdcurses` library used in the original and instead went with the [Bevy engine](https://bevyengine.org). This proved great for learning about the ECS paradigm, but it also made a [web build](https://aleksa2808.github.io/ascii-bomb-ecs/) possible!

## Build

### Native

From the root folder run:

```bash
cargo run --release
```

### Web

From the root folder run:

```bash
wasm-pack build --target web --release
```

Then move the contents of `web` and the `assets` folder into `pkg`. After that, from the `pkg` folder you can start a local server by running:

```bash
# if basic-http-server is not yet installed
cargo install basic-http-server

basic-http-server
```

After that the game should be accessible on `localhost:4000`.

## Bevy usage note

While I am mostly satisfied with how the Bevy engine has been used in the project, a particular thing to note is the extensive usage of "exclusive systems". This has been done in order to ensure that, when some system B runs after some system A, system B will see all of the world changes introduced by system A. I believe the current solution is an antipattern, however at the time it was the most straightforward one to implement since the reduced performance should not be a problem for this particular game.

## Battle mode showcase

![Battle mode gif](doc/battle_mode.gif)

## Screenshots

### Main menu

![Main menu](doc/main_menu.png)

### Story mode

![Story mode #1](doc/story_mode1.png)

![Boss room](doc/boss_room.png)

![Story mode #2](doc/story_mode2.png)

### Battle mode

![Battle mode #1](doc/battle_mode1.png)

![Leaderboard](doc/leaderboard.png)

![Battle mode #2](doc/battle_mode2.png)

### Mobile controls

![Mobile controls](doc/mobile.png)