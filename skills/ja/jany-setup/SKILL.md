---
name: jany-setup
description: jany を初めて使えるようにする。どのシェルで使うかを聞いて、そのシェルの rc に `jany --init` の行を書き、組み込み定義とスキルを置く。zsh なら `jany --on`(jany を付けずに打てる)を rc に書くかも聞く。`/jany-setup` で呼ぶ。「jany をセットアップして」「jany の初期設定」と言われたときにも使う。
---

# jany-setup — jany の初回セットアップ

jany は、ばらばらの語をコマンド行に直して、シェルの入力行に置く CLI。入力行に置くのはシェルのラッパー関数の仕事なので、rc に `jany --init <shell>` の行が要る。このスキルはそれを書く。

**聞くことは AskUserQuestion で聞く**(シェルと `jany --on`)。勝手に決めない。
rc を書き換える前に、今の中身を読む。同じ行がもうあれば足さない。

## 手順

### 1. jany があるか

```
jany --version
```

無ければ止めて、`cargo install jany` を案内する(このスキルでは入れない)。

### 2. シェルを聞く

`echo $SHELL` でログインシェルを見て、それを推奨にして AskUserQuestion で聞く。選択肢は次の 3 つ。

| シェル | rc | 書く行 |
|---|---|---|
| zsh | `~/.zshrc` | `eval "$(jany --init zsh --locale ja)"` |
| bash | `~/.bashrc` | `eval "$(jany --init bash --locale ja)"` |
| fish | `~/.config/fish/config.fish` | `jany --init fish --locale ja \| source` |

- 推奨の選択肢を先頭に置き、ラベルの末尾に「(Recommended)」を付ける
- 説明に書き添えること: 薄い候補・`jany --on`・autorun は zsh だけ。bash のラッパーは作者の手元で確かめていない
- `--locale ja` は、`--init` がシェルを開くたびにスキルを書き直すため。付けないと英語版に戻る

### 3. rc に書く

- rc が symlink なら実体を書き換える(`ls -l` で見る)。rc が無ければ作る(fish は `~/.config/fish/` も)
- `jany --init` を含む行がもうあれば、書き足さない。`--locale` が無い、シェルが違うなど中身が食い違うときは、そのままにして報告に書く
- 無ければ末尾に足す:

```
# jany: put the assembled command on the next prompt
<2 の表の行>
```

### 4. 一度動かして、定義とスキルを置く

rc の行は次にシェルを開いたときに効く。定義とスキルは今置いておく:

```
jany --init <shell> --locale ja >/dev/null
```

stderr の `jany: installed …` を報告に使う。rc の構文も確かめる(`zsh -n ~/.zshrc`、`bash -n ~/.bashrc`、`fish -n ~/.config/fish/config.fish`)。

### 5. zsh なら `jany --on` を聞く

zsh のときだけ。AskUserQuestion で、シェルを開くたびに `jany --on` にするかを聞く。

- 「rc に書く (Recommended)」: `find empty folders` のように `jany` を付けずに打てる。Enter のとき、jany に定義のあるコマンドで始まって後ろに語が続く行は jany を通る。`-` で始まる語・パイプ・リダイレクトを含む行、コマンド単体(`find`)、定義の無いサブコマンド(`docker ps`)は打ったとおりに走る。**`pnpm install` のように `-` を付けない普段の打ち方も jany に回る**ことを書き添える
- 「書かない」: 使いたいときにそのシェルで `jany --on` と打つ。`jany --off` で止まる

「rc に書く」なら、3 で書いた行のすぐ後ろに足す(もうあれば足さない)。起動のたびに出る説明文は捨てる:

```
jany --on 2>/dev/null
```

bash / fish には `jany --on` が無い。聞かずに、無いことだけ報告に書く。

### 6. API キーがあるか

jany は、規則で決まらない語があると jev(OpenRouter)を 1 回呼ぶ。キーの中身は読まない・出さない。あるかどうかだけを見る:

```
[ -n "$OPENROUTER_API_KEY" ] && echo env
grep -l '^api_key' ~/.config/jany/config.toml ~/.config/jurl/config.toml ~/.config/jind/config.toml 2>/dev/null
```

どちらも無ければ、ユーザーに `! jany --setup` を打ってもらう(キーは隠して入力するので、端末が要る。エージェントからは打てない)。キーを会話に貼らせない。

### 7. 報告

- 選んだシェルと、書き換えた rc(実体のパス)と足した行。足さなかった行はその理由
- `jany --on` を rc に書いたか
- 置いた定義とスキル(4 の出力)
- API キーがあるか。無ければ `! jany --setup`
- 今のシェルには効いていない。`exec zsh`(bash なら `exec bash`、fish なら `exec fish`)で開き直す
- 試す例: `jany find log files older than 7 days`(on にしたなら `find log files older than 7 days`)→ 次のプロンプトに `find . -type f -iname '*.log' -mtime +7` が載る
- 次の一歩: よく打つコマンドの定義を足すなら `/jany-register <name>`
