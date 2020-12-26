Tools for viewCount * length of video ranking
====

人類がその動画にかけた時間のランキング集計のために作成したツール

## Index
- [get-nico-data](#get-nico-data)

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

[snapshot-v2-api]: https://site.nicovideo.jp/search-api-docs/snapshot
[SMILEVIDEO-wikipedia]: https://ja.wikipedia.org/wiki/SMILEVIDEO
