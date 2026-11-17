# 設計メモ

## 技術選定

参照元 `coffee-clock-view` の3リング時計の表示を踏襲しますが、時刻の計算・粒子の生成と移動・Canvas 描画は Rust で実装します。Rust を `wasm-pack --target web` で WebAssembly に変換してブラウザに読み込み、UI の配置は HTML と CSS で行います。JavaScript は `src/main.js` の初期化／イベントハンドラだけで、時計ロジックを重複実装しません。

```text
index.html                 画面の構造（Pages のルートエントリ）
Cargo.toml                 Rustクレートと依存関係
Cargo.lock                 Rust依存関係を固定するロックファイル（Git管理対象）
src/lib.rs                 Rustの時計ロジック・Canvas描画・単体テスト
src/main.js                wasmモジュール読み込みとブラウザの描画ループ
src/styles.css             レイアウトと文字組み
scripts/build.sh           wasm-packと静的アセットのビルド
dist/                      ビルド生成物（Git管理対象外）
.github/workflows/         CI・Pagesデプロイ
```

## 時計の構造

- 時刻は閲覧者の端末のローカルタイムゾーンで取得します。
- 時・分・秒のリングはミリ秒を含めた時刻から進捗率を計算し、Canvasに円弧として描きます。
- 初回のフレームでは粒子を出さず、以降は分の切り替わりごとに1回だけ60個を生成します。
- 粒子の位置と寿命には前フレームからの経過ミリ秒を利用し、経過は最大50msに制限します。
- Canvas内部のピクセル解像度は `devicePixelRatio` の最大2倍まで。描画座標はCSSピクセルです。
- 非表示タブでは `requestAnimationFrame` を止め、表示復帰時は前フレーム時刻をリセットします。
- `prefers-reduced-motion: reduce` 有効時は粒子／残像を無効にし、秒の進捗は秒単位の表示にします。

## 責務

- **純粋な Rust 関数**：時刻成分、色相、リングの中心・半径、粒子移動、分の境界判定、表示フォーマット。ブラウザを使わず `cargo test --lib` で確認できます。
- **`ClockApp` (Rust)**：DOM要素とCanvasコンテキストを保持し、`resize`, `frame`, `pause` をWebAssembly経由で公開します。
- **`main.js`**：wasmモジュールの初期化、`requestAnimationFrame`、画面リサイズ・タブ可視性イベントをつなぎます。

現状では Rust を単一 `lib.rs` に置きます。肥大化した場合に `time.rs`、`render.rs` 等に分割します。

## ビルド

`bash scripts/build.sh` は `wasm-pack build --release --target web --out-dir dist/pkg -- --locked` を実行し、`dist/app.js` と `dist/styles.css` を追加します。WebAssembly ローダーは `dist/app.js` に対して相対パスで参照するため、本番用パス `/` とプレビュー用 `/develop/` の両方で同じファイル構成を使えます。

`file://` ではES modules・wasmの読み込みに制約があるため、ローカルはHTTPサーバーを使用します。

## 品質チェックと再現性

- `Cargo.lock` は必ずコミットし、`Cargo.toml` と合わせて管理します。依存関係を変更するときだけ `cargo update` などで更新します。
- `cargo fmt --all -- --check` はRustソースの整形違反を検出します。修正時はローカルで `cargo fmt --all` を実行します。
- `cargo clippy --locked --all-targets -- -D warnings` は全ターゲットのlint警告をエラーとして扱います。
- `cargo test --locked --lib` とWebAssemblyビルドもロックされた依存関係で実行します。依存解決によってロックファイルの更新が必要になった場合は失敗します。

## CI・公開パイプライン

- `.github/workflows/validate-develop.yml`：developへのpush、main/develop宛PRでrustfmt・clippy・Rust単体テスト・WebAssemblyビルドを実行。PRでは公開しません。
- `.github/workflows/deploy-pages.yml`：mainへのpush、検証成功したdevelopのpushを受けて公開します。検証失敗・PRからの結果では公開しません。
- `workflow_run` イベントの場合は検証済みのdevelopコミットSHAをcheckoutし、mainとdevelopの双方を再テスト・ビルドします。
- GitHub Pagesは **`github-pages` 環境1つのみ**。公開ワークフローはmain側から実行します。
- `publish/index.html`, `publish/dist/`, `publish/develop/index.html`, `publish/develop/dist/` を1つのアーティファクトとして公開します。
- 公開ジョブは直列実行し、検証ジョブは古い実行をキャンセルします。

公開URL：
- main：`/rust-clock-view/`
- develop：`/rust-clock-view/develop/`

GitHub Settings → Pages → Source はGitHub Actionsにします。`workflow_run` 起動にはデフォルトブランチの `deploy-pages.yml` が必要です。

## リポジトリ直下のファイル

- `index.html` はルートURLのGitHub Pages配信に必要なので `src/` へ移しません。
- `Cargo.toml` はRust依存関係、`Cargo.lock` はバージョン固定、`scripts/build.sh` はビルド方式を定義します。
- `LICENSE` は公開リポジトリのライセンス、`README.md` はセットアップ、`docs/` は詳細設計です。
