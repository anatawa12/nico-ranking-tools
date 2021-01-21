Tools for ranking of the sum time humanity has watched a video.
====

人類がその動画にかけた時間のランキング集計のために作成したツール

## Index
- [get-nico-data](#get-nico-data)
- [sort-ranking](#sort-ranking)
- [html-gen](#html-gen)
- [集計ステップ](#集計ステップ)

## get-nico-data

[スナップショット検索API][snapshot-v2-api]の結果をローカルに保存するツール

### 使い方

```
get-nico-data [OPTIONS]

OPTIONS:
    -c, --content-id-out <contents-id-out>    file to write contents id proceed.
    -d, --duration <duration>                 duration to be got at a time. defaults 1 week
    -o, --out <out-to>                        file to write to. defaults stdout
    -s, --since <since>                       the begin date of find range. defaults the date starts SMILEVIDEO,
                                              2020/03/06
    -u, --until <until>                       the last date of find range. defaults now
```

引数なしの場合、SMILEVIDEO[(wikipedia)][SMILEVIDEO-wikipedia]の開始日時である
2007年3月6日から現在までのデータを取得する

引数を指定する場合、範囲のはじめ及び終わりをそれぞれ``yyyy/mm/dd``形式で指定し、
一度に取得する範囲を``1week``などの形式で指定する。

### 出力

標準出力または`-o`で指定したファイルに.binを生成

## sort-ranking

指定されたパラメータを使用したランキングを生成する。

### 使い方

```
sort-ranking <input bin> <output bin> <ranking-type>
```

### オプション

- `ranking-type`: ランキングの種類
  - `watch-sum`: 再生回数*再生時間 のランキング
  - `watch-cnt`: 再生回数 のランキング
  - `watch-lng`: 再生時間 のランキング

### 出力

ソート済のbinが生成される

## html-gen

csvからランキングの五番状に表示するhtmlを生成する

### 使い方

```
html-gen <input bin> <output dir>
```

### 出力

`ranking_x.html`が生成される。

## 集計ステップ

1. get-nico-dataで取得
3. sort-rankingでソートする
4. html-genで生成する
