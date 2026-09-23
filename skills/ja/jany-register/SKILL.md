---
name: jany-register
description: jany にコマンド定義(schema.toml / assemble.sh / cases.toml)を追加する。`/jany-register <name> [sub]` で、そのコマンドの「よく使う 8 割」の語を役割に落とし、cases が `jany --test` で全部通るまで直す。
---

# jany-register — jany にコマンドを 1 つ追加する

jany は「崩れた語の並び → 1 本のコマンド行」を、コマンドごとの定義ファイル 3 つで動かす。
このスキルはその 3 つを書いて、`jany --test <name>` が通るところまで持っていく。

- DSL の全キーと assemble の契約: [reference.md](reference.md)(**必ず先に読む**。schema は未知のキーをエラーにする)
- 実例: `examples/find/`(jind 相当。amount・claim・dangerous/preview・pipe)、`examples/curl/`(jurl 相当。next・once・regex・join・pair・mask・scope=command)
- 置き場所: `~/.config/jany/cmd/<name>[/<sub>]/`(`JANY_CMD_DIR` があればそこ)。このスキル自身が雛形を作る

## 手順

### 1. 対象を決める(聞く前に調べる)

- `<name>` のバイナリの `--help` / man を読み、**よく使うオプション 8 割**を選ぶ。全部は載せない(jind が `-mtime` だけに割り切ったのと同じ)。載せないものは `--` の後ろに素通しする
- 役割集合が閉じないもの(任意の SQL 文、jq の式、ffmpeg のフィルタ)は対象外。無理に役割にしない
- 実際に打ちそうな語の並びを **10 本以上**書き出す。崩れ方(語順違い、typo、単位の分離、`8080:80` のような結合語)も入れる。これが cases になる
- サブコマンドがあるなら階層にする: `/jany-register docker run` → `cmd/docker/run/`

### 2. 雛形を置く

`JANY_CMD_DIR` があればそれを、無ければ `~/.config/jany/cmd/` を定義ルートにする。`<name> [sub]` をパス要素として結合し、対象ディレクトリを作る。既存の `schema.toml` がある場合は上書きせず、そこで止める。

このスキルの `template/schema.toml`、`template/assemble.sh`、`template/cases.toml` を対象ディレクトリへコピーして雛形にする。`__NAME__` をコマンド名(サブコマンドを空白で結合したもの)、`__ARGV0__` を assemble の argv 先頭の JSON 配列に置換する。`assemble.sh` には実行権限を付ける。ファイル操作は利用環境の通常の編集手段で行ってよい。

`jany --register` は呼ばない。このスキル単体で、雛形作成から定義の完成まで行う。

### 3. schema.toml を書く

順番はこの通りに考える:

1. **roles**: 語に付く役割を 10〜20 個。`jev` の説明文は英語で具体例つき(jev はこれだけを見て選ぶ)。秘密が入る役割には `mask`
2. **tables**: 語 → 補正値(`file`/`files`/`f` → `f`)。typo 補正の候補にもなる
3. **amount**: 数値語があれば単位表。次元は `time` / `size` / `count`
4. **rules**: 上から順。確実なものから(フラグ、`=` や `:` を含む語、パス、glob、表の語)。**規則で決まる語は jev を呼ばない**ので、迷わない語は全部規則にする。2 語で 1 つ(`-p 8080:80`)は `next`
5. **questions**: 規則で決まらない語に jev へ聞くこと。`role.i` は自動。typo 補正、単位、向き、結合(`join`)など
6. **repair**: 隣接関係(単位を数に付ける、marker が隣を取る、key/value の交互配置)
7. **confirm / defaults**: 破壊的なら `preview_readonly`、取り消しにくいなら `unsafe_note`

### 4. assemble.sh を書く

- stdin JSON → stdout JSON。bash + jq でよい(examples を真似る)。`chmod +x`
- argv の先頭(バイナリ名)もここで決める
- 破壊的な操作は `risk = "dangerous"` と、破壊せず対象だけ見せる `preview` を返す。取り消しにくいものは `"unsafe"`
- 組み立てられない組み合わせは `error` に理由を書く(argv を捏造しない)
- **jq の `//` は `false` を落とす**。真偽値は `== true` で見る

### 5. cases.toml を書いて回す

```
jany --test <name> [sub]
jany --test <name> [sub] --explain      # 語ごとの役割表を見る
```

- 1 で書き出した語の並びを全部 cases にする。規則だけで決まるものは `no_jev = true`
- jev の答えは `[case.jev]` に Mock で書く。**聞かれないキーに答えると失敗する**ので、`--explain` で何が聞かれるか見てから書く
- cases の `words` は `jany <name> --hint` の例としてそのまま人に見せる(`error` / `setup` / `defaults` 付きの case と、先頭 8 本より後は出ない)。**よく使う言い方を先頭に**、人が打ちそうな語順で書く。`[[roles]]` の `jev` の説明も `--hint` に出る
- 全部通るまで schema / assemble を直す。**cases 無しの assemble.sh は信用しない**(LLM が書くと `//` の罠のような間違いが入る)
- 通ったら実機で 2〜3 本: `jany <name> …`(jev を実際に呼ぶ。`OPENROUTER_API_KEY` が要る)。stdout に出た行が意図通りか見る。jany は実行しないので安全

### 6. 報告

- 載せたオプション / 載せなかったオプションと理由
- cases の本数と結果(通った数・落ちた数を出力ごと)
- jev の答えが割れそうな語(裸の語が 2 つの役割で迷う等)

## 注意

- 事実と推測を分ける。バイナリのオプション名は `--help` に当てる。確かめていないことは「確かめていない」と書く
- 頼まれていないコマンドまで登録しない
- `~/.config/jany/cmd/` の既存定義を上書きしない。既存の `schema.toml` があれば、変更せずにユーザーへ報告する
