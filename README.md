# 💣 ascii-bomb-ecs

A Rust remake of the [ascii-bomb](https://github.com/aleksa2808/ascii-bomb) game, built with the [Bevy](https://github.com/bevyengine/bevy) engine.

🎮 **[Play it now in your browser!](https://aleksa2808.github.io/ascii-bomb-ecs/)**

🌐 **[Try out the multiplayer version!](https://github.com/aleksa2808/ascii-bomb-ecs-mp)**

## 🚀 Features

- **Story Mode**: Battle through three different areas with unique boss encounters
- **Battle Mode**: Compete in fast-paced matches against up to 7 other AI players
- **Cross-Platform**: Native desktop and web browser support
- **Mobile-Friendly**: Touch controls optimized for mobile devices
- **???**: Press F to pay respects... and discover something hidden away in the controls menu.

## 🛠️ Build

### Native

From the root folder run:

```bash
cargo run --release
```

### Web Build

1. From the root folder build the WebAssembly package:
   ```bash
   wasm-pack build --target web --release
   ```

2. Prepare the web assets:
   ```bash
   # Copy web files and assets to the pkg directory
   cp -r web/* pkg/
   cp -r assets pkg/
   ```

3. Serve locally:
   ```bash
   # Install the server if you haven't already
   cargo install basic-http-server
   
   # Start the server from the pkg directory
   cd pkg && basic-http-server
   ```

4. Open your browser and navigate to `http://localhost:4000`

## 🎮 Gallery

### Battle Mode in Action
![Battle mode showcase](doc/battle_mode.gif)

### Game Screens

<details>
<summary>📸 View Screenshots</summary>

#### Main Menu
![Main menu](doc/main_menu.png)

#### Story Mode
![Story mode gameplay](doc/story_mode1.png)

![Epic boss battle](doc/boss_room.png)

![Story progression](doc/story_mode2.png)

#### Battle Mode
![Battle mode action](doc/battle_mode1.png)

![Global leaderboard](doc/leaderboard.png)

![Intense battles](doc/battle_mode2.png)

#### Mobile Experience
![Mobile touch controls](doc/mobile.png)

</details>
