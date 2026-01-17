# gemini

Reversi (Othello) project.

## Benchmarks

`gemini_core` クレートのベンチマークは criterion を使って実装しています。

### 実行例

```bash
# workspace 全体のビルド/ベンチ
cargo bench -p gemini_core

# 個別ベンチ
cargo bench -p gemini_core --bench engine
cargo bench -p gemini_core --bench ai_random
cargo bench -p gemini_core --bench ai_alphabeta
```

結果は標準で `target/criterion/` に出力されます。

## UI

### SDL

```bash
# Ubuntu
sudo apt install libsdl2-dev

# MacOS
brew install sdl2
```

```bash
cargo run -p gemini_sdl
```

### WASM

`gemini_wasm` クレートは `wasm32-unknown-unknown` を想定しています。

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Develop:

```sh
wasm-pack build ./gemini_wasm --release --target web --no-pack --no-typescript
python3 -m http.server 8080 --directory ./gemini_wasm
```

ブラウザで `http://localhost:8080/index.html` を開くとリバーシで遊べます。
