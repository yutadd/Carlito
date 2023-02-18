# Carlito

![commit_frequence_badge](https://img.shields.io/github/commit-activity/w/yutadd/Carlito)
![last_commit](https://img.shields.io/github/last-commit/yutadd/Carlito)
![wip](https://img.shields.io/badge/work%20in%20progress-wip-green)

## このブロックチェーンについて(計画中)



このブロックチェーンは個人開発の、企業向けを想定している PoA 方式のブロックチェーンです。  
汎用的で強力なブロックチェーンとしてApacheやMysqlなどのように「簡単に使えるバックエンド」の一つとなることを目指しています！
使用方法の例は 使い方の例(計画中) を御覧ください。

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
### それぞれの仕組み
ネットワーク  
![network graph](https://github.com/yutadd/Carlito/blob/master/images/Carlito.png)  
認証  
![handshake graph](https://github.com/yutadd/Carlito/blob/master/images/Carlito2.png)


## 実行方法

### シュミレーション

docker compose を用いて、DNSと3つのノードが存在するCarlitoネットワークをシミュレーションすることができます！  
Carlitoのrootで個のコマンドを実行することにより、しばらくすると完了します！  
```
docker compose up
```



### 本番環境(計画中)

デプロイされているネットワークに接続します。

Config/config.ymlから各種設定を行い、./release.shを実行することで、ネットワークへ接続できるように実装する予定です。

```
./release.sh
```

## 使い方の例(計画中)

テキスト形式のデータを保存できるため、スクリプトとしてデータを保存することで、用途に合わせて様々な表現が可能。

例えば

```
ADD '7a62ec......f10'
READ '7a62ec......f10' User01
ADD '94c7ed......834'
DEL '7a62ec......f10'

```

こういったログをこのブロックチェーンに保存していき、フロントエンドで現在の状態を計算することで信頼の置けるファイル操作・閲覧ログとしての使用法ができる。

※ADD がファイルの追加を表し、READ がファイルの読み取り（第 2 引数はユーザー名）DEL は削除を表すなどのルールをもとに、作成し、オブザーバープログラムを作成して操作が起きるたびに操作内容をブロックチェーンに追記していく。
