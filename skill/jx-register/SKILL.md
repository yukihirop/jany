---
name: jx-register
description: jx にコマンド定義(schema.toml / assemble.sh / cases.toml)を追加する。`/jx-register <name> [sub]` で、そのコマンドの「よく使う 8 割」の語を役割に落とし、cases が `jx test` で全部通るまで直す。
---

# jx-register — jx にコマンドを 1 つ追加する

jx は「崩れた語の並び → 1 本のコマンド行」を、コマンドごとの定義ファイル 3 つで動かす。
このスキルはその 3 つを書いて、`jx test <name>` が通るところまで持っていく。

- DSL の全キーと assemble の契約: [reference.md](reference.md)(**必ず先に読む**。schema は未知のキーをエラーにする)
- 実例: `examples/find/`(jind 相当。amount・claim・dangerous/preview・pipe)、`examples/curl/`(jurl 相当。next・once・regex・join・pair・mask・scope=command)
- 置き場所: `~/.config/jx/cmd/<name>[/<sub>]/`(`JX_CMD_DIR` があればそこ)。`jx register <name> [sub]` が雛形を置く

## 手順

### 1. 対象を決める(聞く前に調べる)

- `<name>` のバイナリの `--help` / man を読み、**よく使うオプション 8 割**を選ぶ。全部は載せない(jind が `-mtime` だけに割り切ったのと同じ)。載せないものは `--` の後ろに素通しする
- 役割集合が閉じないもの(任意の SQL 文、jq の式、ffmpeg のフィルタ)は対象外。無理に役割にしない
- 実際に打ちそうな語の並びを **10 本以上**書き出す。崩れ方(語順違い、typo、単位の分離、`8080:80` のような結合語)も入れる。これが cases になる
- サブコマンドがあるなら階層にする: `jx register docker run` → `cmd/docker/run/`

### 2. 雛形を置く

```
jx register <name> [sub]
```

3 ファイルの雛形が置かれる。既にあれば上書きしない。

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
jx test <name> [sub]
jx test <name> [sub] --explain      # 語ごとの役割表を見る
```

- 1 で書き出した語の並びを全部 cases にする。規則だけで決まるものは `no_jev = true`
- jev の答えは `[case.jev]` に Mock で書く。**聞かれないキーに答えると失敗する**ので、`--explain` で何が聞かれるか見てから書く
- 全部通るまで schema / assemble を直す。**cases 無しの assemble.sh は信用しない**(LLM が書くと `//` の罠のような間違いが入る)
- 通ったら実機で 2〜3 本: `jx <name> …`(jev を実際に呼ぶ。`OPENROUTER_API_KEY` が要る)。stdout に出た行が意図通りか見る。jx は実行しないので安全

### 6. 報告

- 載せたオプション / 載せなかったオプションと理由
- cases の本数と結果(通った数・落ちた数を出力ごと)
- jev の答えが割れそうな語(裸の語が 2 つの役割で迷う等)

## 注意

- 事実と推測を分ける。バイナリのオプション名は `--help` に当てる。確かめていないことは「確かめていない」と書く
- 頼まれていないコマンドまで登録しない
- `~/.config/jx/cmd/` の既存定義を上書きしない(`jx register` は既存があれば止まる)
