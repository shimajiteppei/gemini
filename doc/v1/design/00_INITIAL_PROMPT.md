# ディレクトリ構成

- gemini_core
  - src
    - engine
    - ai
      - random
      - alphabeta
  - benches
    - engine
    - ai
      - random
      - alphabeta
  - tests
- doc
- gemini_sdl
- gemini_wasm

# 必須要件

- Rustでリバーシを実装
- https://github.com/shimajiteppei/aries を参考に、gemini_core, gemini_sdl, gemini_wasmでロジックとUIを分離して実装
- gemini_coreロジックは、ゲームを管理するengineと、AIを管理するaiを分離して実装
  - engineの実装は以下の方針
    - 盤面はu64のビットボード
    - 反転と合法手判定はKogge-Stone法
  - aiのアルゴリズムは2種類実装
    - 合法手からランダムに打つ。乱数は固定値seedの線形合同法で生成
    - アルファベータ法
  - engineとaiのベンチマークを取得
  - 結合テストをgemini_core/testに実装
    - CPU同士の対戦
      - random vs random
      - random vs alphabeta
- UIはSDL, WASMで実装
  - 人間/AI両方の手番が終わるたびに描画を更新。AI手番は0.3秒遅延させる
  - WASMの実装にはHTML canvas APIを使用

# 実装指示
- AWS Kiroのようにステップを分けて設計・実装
- doc/v1/designに実装ステップを計画したmarkdownファイルを作成
- 各ステップでclippyを通す
- clippyのルールは編集不可。allowによる回避も不可。clippyのルール同士に矛盾がある場合は私に判断を質問すること
- コンパイル、clippy、テストのエラーはファイルに出力して修正する
- clippyのエラーは`cargo clippy --fix --allow-dirty`で修正してもいい
- モジュール定義に`mod.rs`は禁止。self named moduleのみ許可
- 1ファイルが500行を超える場合はファイル分割推奨。1000行を超える場合はファイル分割必須
