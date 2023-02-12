use mods::certification::key_agent;
use mods::certification::sign_util;
use mods::network::connection_listener;
use mods::network::dns_seed;
use std::io::stdin;
use std::thread;
mod mods;
/**
# 信用
## 前提
各企業とノードとキーペアは1対1対1で対応するように事前に生成し、trusted_hostsに公開鍵を登録しておく
ネットワークへ接続するには認証を要求する。
### 可用性
前提より、会社の信頼を掛けてノードを運営することになるため、ノードの可用性を維持する
モチベーションとなる。
### 完全性
ノードは決められたタイミングで正しいブロックを生成しなければ、ネットワークから排除され、
会社の信用も失う可能性があるため、ノードは正しいタイミングで正しいブロックを生成しようという動機が生まれる。

# 信用の追加方法
いつからいつまで信頼していたかを明らかにするため、ブロックチェーンに承認情報を記述する。
追加する手順は以下の通り。
認証されたノードが認証ノード追加の提案を作成し同定案内で次のsignerを指定し、署名し、ブロードキャストする。
その後、指定されたノードは、ノードの管理者に決定を促し、承認する場合、次のノードを指定し、
自分の署名を追加する。
追加された署名が信頼されたノードの半数を超えたら、ブロックチェーンに追記する。
これにより、途中参加するノードも、受け取ったブロックチェーンからブロックを正しく検証することができる。

# ブロック生成者算出手順
trusted_hostsに記載されている順番に0より、担当していく。
最後尾に到達したら、また0から担当させる。
担当者がマイニングしない限り、ブロックが生成されないため、
「ネットワーク参加者は、同じIDで複数のノードを実行することにより、冗長性を展開」*ref1 する必要がある。
それぞれのノードは、ブロックが生成され、送られてきた時間と、生成されているべき時間の差を記録しておき、
ブロックの生成が頻繁に滞るノードがあれば、記録より滞りやすいノードを排除すべきか検討し、
信用の追加方法を応用して、排除を行う。

#ネットワークの対障害性・対改ざん性
・担当のノードがストップした場合、ネットワークのマイニングはストップしてしまう。
そのため、同一の鍵を持つノードを複数参加させて可用性を挙げられる。
・一つのノードが乗っ取られた場合、乗っ取ったノードに対して送られてきたトランザクションを恣意的に
拒否することは可能になり、そうするとユーザーは
乗っ取られていないノードがマイニングの担当になるのを待つ必要がある。
# ノードが乗っ取られた場合、攻撃者ができること
マイニングにて、トランザクションの選別を行う
大量のトランザクションを含んだブロックを生成する。
ブロック生成担当の無視


DEV
PoAはValidatorの人数がわかっているため、多数決も可能。
多数決で過去のチェーンにロールバックすることもできる可能性がある。

# TODO
user.rsで認証の実装
dockerでネットワークシュミレーションを行えるようにする。

#ref
ref1: https://morioh.com/p/de3cf10c2194

*/
fn main() {
    println!("Initializing...");
    key_agent::init();
    sign_util::init();
    thread::spawn(|| {
        connection_listener::run();
        println!("thread-Inited");
    });
    dns_seed::init();
    println!("Inited");
    loop {
        let line = &mut String::new();
        let size = stdin().read_line(line).unwrap();
        if size > 0 {
            println!("your input:{}", line);
        }
    }
}
