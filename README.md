# ascii-bomb-ecs

Port of the [ascii-bomb](https://github.com/aleksa2808/ascii-bomb) game in Rust using the Bevy engine. Available as a [web build](https://aleksa2808.github.io/ascii-bomb-ecs/)!

#### UPDATE: Now there is also an [online multiplayer version](https://github.com/aleksa2808/ascii-bomb-ecs-mp)!

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
