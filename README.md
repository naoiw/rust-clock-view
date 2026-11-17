# Rust Clock

Rust と WebAssembly、HTML Canvas で作ったブラウザ時計です。参照元の [Coffee Clock](https://github.com/naoiw/coffee-clock-view) と同じ体験を、時計のロジックと描画を Rust で実装して再現します。

## 主な機能

- 時・分・秒を3本のリングで表示（滑らかな進捗）
- 時刻に合わせて変化する配色
- 毎分の切り替わりに発生する粒子エフェクト（初回表示では発生しない）
- デジタル時刻・日本語の日付の表示（閲覧者のローカルタイム）
- スマートフォン・高解像度ディスプレイ・OSの「動きを減らす」設定に対応
- タブが非表示の間は描画を停止

## 公開先

| 環境 | ブランチ | URL |
| --- | --- | --- |
| 本番 | `main` | https://naoiw.github.io/rust-clock-view/ |
| 開発プレビュー | `develop` | https://naoiw.github.io/rust-clock-view/develop/ |

本番とプレビューは1つの GitHub Pages サイト（`github-pages`環境）内の別々のパスで公開します。初回はリポジトリの Settings → Pages → **Source: GitHub Actions** に設定してください。

## 技術スタック

- **Rust** — 時刻計算、リング・粒子の Canvas 描画、画面ラベル
- **wasm-bindgen / web-sys / js-sys** — Rust とブラウザ API の接続
- **WebAssembly** — ブラウザでの実行
- **HTML / CSS / 少量の JavaScript** — 画面の構造・スタイル・起動／イベント制御
- **wasm-pack / Cargo** — ビルド・テスト（`Cargo.lock` で依存バージョンを固定）
- **rustfmt / clippy** — フォーマット・静的解析
- **GitHub Actions / GitHub Pages** — CI・本番・開発プレビュー

## ローカル開発

Rust と `wasm-pack`、Python 3 を用意します。

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --lib
bash scripts/build.sh
python3 -m http.server 3000
```

`http://localhost:3000/` を開いてください。ブラウザの ES module / WebAssembly のため、`file://` での直接起動はできません。変更後は `bash scripts/build.sh` を再実行します。生成物 `dist/` は Git 管理対象外です。`Cargo.lock` はアプリの依存関係を固定するため Git 管理します。ビルドとテストは `--locked` により、ロックファイルとの不一致で失敗する構成です。依存関係を意図的に更新するときのみ `cargo update` を実行し、`Cargo.lock` の差分をコミットします。

## ブランチ運用

1. 通常の実装・修正は `develop` で行い、`main` は本番用として維持します。
2. Pull Request では rustfmt・clippy・Rust のテストと WebAssembly ビルドを実行し、デプロイは行いません。
3. `develop` への push は検証成功後、`main` 側のワークフローから開発プレビューに反映します。
4. 本番反映は **`develop` → `main` の Pull Request** で行います。マージ後も `develop` は削除しません。

## ディレクトリ構成

- `Cargo.toml` — Rust クレートと依存関係
- `Cargo.lock` — Rust 依存パッケージのバージョン固定
- `src/lib.rs` — 時計の計算、Canvas 描画、ブラウザ公開 API、単体テスト
- `src/main.js` — WebAssembly の初期化とブラウザのイベント制御のみ
- `src/styles.css` — レイアウト
- `index.html` — 画面のエントリポイント
- `scripts/build.sh` — WebAssembly と静的ファイルのビルド
- `.github/workflows/` — CI・GitHub Pages デプロイ
- `docs/` — 設計ドキュメント

## ドキュメント

- [設計・デプロイ構成](docs/architecture.md)

## ライセンス

[MIT License](LICENSE)
