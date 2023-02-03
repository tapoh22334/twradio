# ついよみ
本ソフトウェアはツイッターのタイムラインを読み上げます。
使用するためには後述の音声合成ソフトが別途必要です。

## 使い方
1. 音声合成ソフトを起動
2. 本ソフトウェアを起動するとブラウザが開く
3. ブラウザ上で本アプリによるTwitterの使用を許可する
4. 読み上げが始まります

## 注意
本アプリケーションはTwitterの仕様変更や規制により突然利用できなくなる可能性があります。

## 音声合成ソフト

以下の音声合成ソフトに対応しています

 - VOICEVOX v0.13.1
 - COEIROINK v1.6.0
 - LMROID v1.3
 - SHAREVOX v0.1.7
 - ITVOICE v1.0.1

他のバージョンでの動作は検証していません

## 機能

- ツイート読み上げ
- 話者変更

## 動作環境

 - Windows 10

Windowsでテストしていますが、
mac, linuxでもビルドできると思います。

## 環境構築

Rust/Tauri + React(フロントエンド)で書かれています。
以下の手順で環境を構築します。

### Rust/Tauriをインストール

https://tauri.app/v1/guides/getting-started/prerequisites/

### npmを準備
```
npm install
```

## ビルド

ディレクトリトップで以下のコマンドを実行します。
'src-tauri/target/'以下に実行ファイルが生成されます。

```
cargo tauri build
```

lint, format

```
npx prettier --write src/*.ts src/*.tsx
(cd src-tauri; cargo fmt)
(cd src-tauri; cargo clippy -- -D warnings)
```

## ライセンス

MIT
