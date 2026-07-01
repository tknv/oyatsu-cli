[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/tknv/oyatsu-cli)  

# oyatsu (CLI版)

おやつの時間を把握する為の 
[Oyatsu (Android版)](https://github.com/tknv/Oyatsu) のロジックを移植した
コマンドラインツール。

和暦・和風月名・和時計時刻(不定時法)・二十四節気・干支をターミナルに表示します。
「未」の刻(八つ時、現代のおよそ13:00〜15:00)には先頭に「おやつ」と表示されます。

## ビルド

Rust (cargo) が必要です。

```sh
cargo build --release
```

`target/release/oyatsu` に実行ファイルができます。`PATH` の通った場所
(例: `~/.local/bin` や `/usr/local/bin`) にコピーすると `oyatsu` コマンドとして使えます。

```sh
sudo cp target/release/oyatsu /usr/local/bin/oyatsu
# もしくは
cp target/release/oyatsu ~/.local/bin/oyatsu
#　もしくは
sudo make install
```

## 使い方

```sh
oyatsu
```

```
令和八年 水無月 丑二つ 夏至 丙午
(日出05:09-日入18:35)
```

おやつ時(未の刻)の場合:

```
おやつ 令和八年 水無月 未二つ 夏至 丙午
(日出05:09-日入18:35)
```

日の出・日の入り時刻は標準エラー出力に補足として表示されます。

```sh
oyatsu 2&> /dev/null   # 補足情報を非表示にする
おやつ 令和八年 水無月 未二つ 夏至 丙午
```

### 位置情報

和時計時刻(不定時法)は緯度経度から日の出・日の入りを計算して求めます。

優先順位:

1. `-lat` / `-lon` オプションで指定した値
2. `~/.config/oyatsu` (1行目に緯度、2行目に経度を記載) Mac: `~/Library/Application Support/oyatsu`, Win: `%APPDATA%\oyatsu`
3. どちらもなければ東京駅 (`35.6895`, `139.6917` 付近) を使用

```sh
# その都度、緯度経度を指定して実行
oyatsu -lat 35.0116 -lon 135.7681

# 設定ファイルを使う場合
mkdir -p ~/.config
cat > ~/.config/oyatsu << EOF
35.0116
135.7681
EOF
oyatsu   # -lat -lon なしで実行できる
```

`-lat`/`-lon` のどちらか一方だけ指定した場合は、もう一方は設定ファイル(あれば)
の値、それも無ければデフォルト(東京駅)の値で補います。

## 仕様

- CUI (ターミナルで動くコマンド)
- 表示項目: 和暦、和風月名、和時計時刻(不定時法)、二十四節気、干支
- 未の刻(八つ時)のときは先頭に「おやつ」を付与
- `~/.config/oyatsu` に緯度・経度があれば `-lat`/`-lon` なしでも実行可能
- `~/.config/oyatsu` がなくても `oyatsu -lat 緯度 -lon 経度` で実行可能

## ライセンス

[GNU GPLv3 or later](http://www.gnu.org/licenses/gpl.html)
(元プロジェクト [Oyatsu](https://github.com/tknv/Oyatsu) のライセンスを継承)

### おやつ

© [Wikipedia, CC BY-SA 4.0](https://ja.wikipedia.org/wiki/%E3%81%8A%E3%82%84%E3%81%A4)  

### 不定時法

© [国立天文台](https://eco.mtk.nao.ac.jp/koyomi/wiki/BBFEB9EF2FC4EABBFECBA1A4C8C9D4C4EABBFECBA1.html)   

### 和風月名

出典：国立国会図書館「日本の暦」 [https://www.ndl.go.jp/koyomi/](https://www.ndl.go.jp/koyomi/)  
![日本の暦](https://www.ndl.go.jp/koyomi/about/img/banner.gif)

### だんご

出典:農林水産省ウェブサイト [農林水産省 うちの郷土料理 串だんご 東京都](https://www.maff.go.jp/j/keikaku/syokubunka/k_ryouri/search_menu/menu/34_29_tokyo.html#:~:text=%E5%9B%A3%E5%AD%90%E3%81%AF%E5%B9%B3%E5%AE%89%E6%99%82%E4%BB%A3%E3%81%AB,%E5%A3%B2%E3%82%8B%E5%BA%97%E3%82%82%E3%81%A7%E3%81%8D%E3%81%9F%E3%80%82)

> 「花より団子」という言葉がはやるほど人気が出て、全国的に広まった一串5つ刺しの串だんごは、京都発祥と言われる。
東京でも、江戸時代では5つ刺しが主流であり、1本5文銭で販売されていた。4文銭が流通を始めてから、買い求める客で混雑する中、**4文銭を置いて持ち帰る不正を行う客が増え、店主が困り苦肉の策でだんごの数を減らして4つ一串にしたことが、4つ刺しのだんごが生まれたはじまり** という記録が残っている。現在でも串だんごは **関東では4つ刺し、関西では5つ刺し** が主流である。


Copyright (C) [2026]  
This work incorporates AI-assisted.  
本作品はAI支援を含みます。  

このプログラムはフリーソフトウェアです。あなたはこれを、フリーソフトウェア財団によって発行されたGNU一般公衆利用許諾書（バージョン3か、それ以降のバージョンのうちどれか）が定める条件の下で再頒布または改変することができます。
このプログラムは有用であることを願って頒布されますが、全くの無保証です。商業可能性の保証や特定目的への適合性は、言外に示されたものも含め、全く存在しません。詳しくはGNU一般公衆利用許諾書をご覧ください。
あなたはこのプログラムと共に、GNU一般公衆利用許諾書のコピーを一部受け取っているはずです。もし受け取っていなければ、https://www.gnu.org/licenses/ をご覧ください。
