Tools for ranking of the sum time humanity has watched a video.
====

人類がその動画にかけた時間のランキング集計のために作成したツール

## Index
- [get-nico-data](#get-nico-data)
- [merge-nico-data](#merge-nico-data)
- [sort-ranking](#sort-ranking)
- [merge-rankings](#merge-rankings)
- [html-gen](#html-gen)
- [集計ステップ](#集計ステップ)

## get-nico-data

[スナップショット検索API][snapshot-v2-api]の結果をローカルに保存するツール

### 使い方

```
get-nico-data
# or
get-nico-data <since> <until> <per>
```

引数なしの場合、SMILEVIDEO[(wikipedia)][SMILEVIDEO-wikipedia]の開始日時である
2007年3月6日から現在までのデータを取得する

引数を指定する場合、範囲のはじめ及び終わりをそれぞれ``yyyy/mm/dd``形式で指定し、
一度に取得する範囲を``1week``などの形式で指定する。

### 出力

カレントディレクトリ以下に次のような形式で出力する
```
cwd
 `--out
     `--yyyy/mm/dd
         +--version.json # snapshotのバージョン
         `--ranking_xxxxxx.json # レスポンスを連番で
```

## merge-nico-data

[get-nico-data] で取得したデータを各週毎に一つのjsonにまとめる

### 使い方

```
merge-nico-data <options> <target directory>
```

`out`ディレクトリを引数に指定する

### オプション

- `-a`: バージョンごとに`merged_xx.bin`をカレントディレクトリに作成する
- `-d`: ranking_xxxx.jsonおよびversion.jsonを削除

### 出力

それぞれのディレクトリに`merged.bin`が作成される。
rankingと同形式にmeta内にlast_modifiedが追加された形のjsonとなる。

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

ranking_counterが指定された`merged.bin`形式

## merge-rankings

`sort-ranking` で生成された複数のランキングバイナリを読み込み、csvに出力する

### 使い方

```
merge-rankings <out csv> <ranking bin files...>
```

### 出力

要素が順に ``rank, ranking key, video id, get at, posted at, view count, video length`` なcsv. (ヘッダ行あり)

[get-nico-data]: #get-nico-data
[snapshot-v2-api]: https://site.nicovideo.jp/search-api-docs/snapshot
[SMILEVIDEO-wikipedia]: https://ja.wikipedia.org/wiki/SMILEVIDEO

## html-gen

csvからランキングの五番状に表示するhtmlを生成する

### 使い方

```
html-gen <input csv> <output dir>
```

### 出力

`ranking_x.html`が生成される。

## 集計ステップ

1. get-nico-dataで取得
2. merge-nico-data -aでまとめる
3. sort-rankingで各週毎にソートする
4. merge-rankingsでcsvにする
