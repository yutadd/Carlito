# Carlito

## このブロックチェーンについて(計画中)

このブロックチェーンは個人開発の、コンシューマ向けを想定している PoA 方式のブロックチェーンです。  
データの保存用途を計画しています。

## RoadMap(計画中)

```
検討及び計画（完了）
↓
プログラム部以外のディレクトリ雛形の作成（完了）
↓
根幹のコード実装（WIP）
    ↕
Dockerによるデバッグ環境の整備（WIP）
↓
RestAPIの作成（未作業）
↓
実証運転＆APIを利用したフロントエンド例やDAPPsの作成（未作業）
```

## 実行方法(計画中)

### シュミレーション

docker compose を用いて、いくつかのノードと DNS が存在するネットワークをシュミレーションすることができます。

```
docker compose up
```

### 本番環境

デプロイされているネットワークに接続します。

詳細は引数リストをご参照ください

```
./Carlito
```

## 引数リスト(計画中)

```
-s DNSシードを決定する。
-v 詳細情報を標準出力する。
-a ネットワークに接続せず、ノードの状態を出力し、終了します。
```

## 使い方の例(計画中)

テキスト形式のデータを保存できるため、スクリプトとしてデータを保存することで、用途に合わせて様々な表現が可能。

例えば

```
ADD '7a62ec......f10'
ADD '94c7ed......834'
DEL '7a62ec......f10'
```

といったイベントをこのブロックチェーンに保存していき、フロントエンドで現在の状態を計算することで信頼の置けるファイル管理ログとしての使用法。
