# ついよみ

ツイッターを読み上げるソフトウェアです。
別途COEIROINKv1.6.0を用意する必要があります。
他のバージョンでは動作の検証を行っていません。

## 実行手順

1. COEIROINKを起動
2. 本ソフトウェアを起動
3. ブラウザが自動的に開く
4. 本アプリがTwitterを使用することを許可する
5. 読み上げが開始される

## 環境

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


## ライセンス

MIT
