# 実装ステップ計画（v1）

本ドキュメントは `doc/v1/design/00_INITIAL_PROMPT.md` の要件を、AWS Kiro のように小さなステップへ分解して進めるための計画です。

## 共通ルール

- 各ステップ完了時点で `cargo clippy --workspace --all-targets` を通す
- Clippy ルール（`core/Cargo.toml` の設定）は編集しない
- 各ステップ完了時点で `cargo test --workspace` を通す（該当クレート/ターゲットが増えるほど重要）

## ステップ 0: 足場の整備

### 目的

- `core` に `engine` / `ai` のモジュール構成を用意し、外部（`sdl`/`wasm`）から呼び出すための最小公開 API を定義する

### 成果物

- `core/src/lib.rs`
- `core/src/engine/mod.rs` ほか
- `core/src/ai/mod.rs` ほか

### 完了条件

- `cargo clippy --workspace --all-targets` が成功
- `cargo test --workspace` が成功

## ステップ 1: `core` のエンジン（ビットボード + ルール）

### 目的

- 盤面を `u64` のビットボードで表現する
- 合法手生成と反転を Kogge-Stone 法で実装する

### 成果物

- `Position`（局面）、`Color`（手番）、`Move`（着手）などの型
- `legal_moves` と `apply_move`（反転含む）
- 終局判定（双方パス）とスコア計算

### 完了条件

- 主要 API が `core` クレート外から使用可能
- `cargo clippy --workspace --all-targets` / `cargo test --workspace` が成功

## ステップ 2: `core` の AI（random）

### 目的

- 合法手からランダムに 1 手選ぶ AI を実装する

### 成果物

- `ai/random` 実装
- エンジン側と接続するための AI トレイト（またはそれに準ずるインターフェイス）

### 完了条件

- 1 手の選択が「合法手のみ」から行われる
- `cargo clippy --workspace --all-targets` / `cargo test --workspace` が成功

## ステップ 3: 結合テスト（CPU 同士対戦）

### 目的

- `core/tests` に CPU 同士の対戦を実装して、ゲームが最後まで進行できることを保証する

### 成果物

- `random vs random`
- `random vs alphabeta`（alphabeta は後続ステップで実装し、テストは先に骨格だけ用意する）

### 完了条件

- テストが安定して完走し、盤面・石数などの不変条件が満たされる
- `cargo clippy --workspace --all-targets` / `cargo test --workspace` が成功

## ステップ 4: `core` の AI（alphabeta）

### 目的

- アルファベータ探索を実装し、一定深さで最善手を選ぶ

### 成果物

- `ai/alphabeta` 実装（探索深さ、評価関数）
- `random vs alphabeta` の結合テストを有効化

### 完了条件

- 合法手のみを探索対象にする
- `cargo clippy --workspace --all-targets` / `cargo test --workspace` が成功

## ステップ 5: ベンチマーク

### 目的

- `engine` と `ai` の性能を計測できるようにする

### 成果物

- `core/bench/engine`（合法手生成/反転など）
- `core/bench/ai/random`, `core/bench/ai/alphabeta`（思考時間など）

### 完了条件

- `cargo clippy --workspace --all-targets` / `cargo test --workspace` が成功
- ベンチ実行手順が README などで再現可能

## ステップ 6: UI（SDL / WASM）

### 目的

- `core` と UI を分離し、SDL と WASM（Canvas API）で操作・描画できるようにする

### 成果物

- `sdl`：盤面描画、クリック入力、手番/終局表示、AI 対戦
- `wasm`：Canvas 描画、クリック入力、WASM 公開 API（初期化/クリック/描画更新）

### 完了条件

- `cargo clippy --workspace --all-targets` / `cargo test --workspace` が成功
