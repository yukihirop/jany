---
name: jany-teardown
description: jany をやめるときに、jany が置いたものを片付ける。rc の `jany --init` / `jany --on` の行と jany を指す alias・compdef、`~/.config/jany/`(定義・設定・API キー)、`~/.agents/skills/` のスキルとその symlink、バイナリ。何を消すかは AskUserQuestion で聞き、消す前に一覧を見せて確認を取る。`/jany-teardown` で呼ぶ。「jany をアンインストールして」「jany をやめたい」と言われたときにも使う。
---

# jany-teardown — jany を片付ける

`/jany-setup` と `jany --init` が置いたものを、置いた逆の順に外す。

**消すものは AskUserQuestion で聞く。** 取り消しが効かないので、消す前に対象の一覧(パスと行)を見せて、もう一度確認を取る。
自分で書いた定義と API キーは、既定では消さずに退避する。

## jany が置くもの

| もの | 場所 | 置いたもの |
|---|---|---|
| rc の行 | `~/.zshrc` / `~/.bashrc` / `~/.config/fish/config.fish` | `eval "$(jany --init …)"`(fish は `jany --init fish \| source`)、`jany --on`、jany を指す alias と `compdef` |
| 定義と設定 | `~/.config/jany/`(`JANY_CONFIG_DIR` / `JANY_CMD_DIR` があればそこ) | `cmd/<name>/`(組み込みの find / curl / docker run と、自分で書いた定義)、`config.toml`(`[jev] api_key` を含みうる) |
| スキル | `~/.agents/skills/jany-{setup,register,update,teardown}/`(`JANY_SKILL_DIR` があればその親) | スキル本体 |
| symlink | `~/.claude/skills/`、`~/.codex/skills/` | 上のスキルを指す symlink |
| バイナリ | `~/.cargo/bin/jany` | `cargo install jany` |

`jind` / `jurl` の `~/.config/{jind,jurl}/` は jany のものではない。触らない。

## 手順

### 1. 見つける

消さずに、在りかだけを集める:

```
command -v jany; jany --version
echo "$JANY_CONFIG_DIR $JANY_CMD_DIR $JANY_SKILL_DIR"
grep -n 'jany' ~/.zshrc ~/.bashrc ~/.config/fish/config.fish 2>/dev/null
jany --list
ls -la ~/.agents/skills ~/.claude/skills ~/.codex/skills 2>/dev/null | grep jany
```

- rc は symlink なら実体を見る(`ls -l`)
- `grep` に出た行のうち、jany のものだけを拾う: `jany --init` の行、`jany --on` の行、値が `jany` か jany を指す alias に行き着く alias(`alias j='jany'`、`alias jpnpm='j pnpm'`)、`compdef _jany_complete …`。コメント行 `# jany: …` も。**jany を含むだけの別の行(パスや別コマンドの引数)は拾わない**。迷う行は報告に回して触らない
- **`grep 'jany'` には、別の alias を通して jany に行き着く alias が出ない**(`alias jpnpm='j pnpm'` には jany の字が無い)。見つけた alias 名(`j` など)で `grep -nE "^alias [^=]+=['\"]?j( |['\"])" <rc>` を探し、新しい名前が出なくなるまで繰り返す。その名前の `compdef … <名前>` も拾う
- 自分で書いた定義: `jany --list` のうち `find` / `curl` / `docker run` 以外
- API キー: `grep -c '^api_key' ~/.config/jany/config.toml`。**中身は読まない・出さない**
- symlink は、リンク先が jany のスキルのものだけ(`readlink` で確かめる)

### 2. 何を片付けるか聞く

AskUserQuestion で、1 の結果を添えて聞く(multiSelect)。

- 「rc の行 (Recommended)」: 1 で拾った行。**これを外さないと、`jany --init` が開くたびにスキルを置き直す**。バイナリを消すなら必須(残すとシェル起動時に `jany: command not found` が出る)
- 「スキルと symlink (Recommended)」
- 「定義と設定(~/.config/jany)」: 自分で書いた定義の名前と、API キーがあるかを書き添える
- 「バイナリ(cargo uninstall jany)」

「定義と設定」を選んだら、続けて AskUserQuestion でやり方を聞く:

- 「退避する (Recommended)」: `~/.config/jany` を `~/jany-backup-<YYYYMMDD>/` に移す。戻したくなったら移し戻せる。API キーもそこに残ることを書き添える
- 「消す」: 自分で書いた定義と API キーも消える

### 3. 一覧を見せて確認する

選ばれたものについて、外す行(ファイルと行番号と中身)、移す・消すパスを全部並べて見せ、AskUserQuestion で「この内容で片付ける」「やめる」を聞く。「やめる」なら何もせずに終わる。

### 4. 片付ける

この順で行う(先にスキルやバイナリを消すと、rc の行が次のシェルでエラーを出す):

1. **rc の行**: 書き換える前に rc を `<rc>.jany-teardown.bak` にコピーする。3 で見せた行だけを消す。消した後に構文を確かめる(`zsh -n ~/.zshrc`、`bash -n ~/.bashrc`、`fish -n ~/.config/fish/config.fish`)。落ちたらバックアップから戻して止まる
2. **定義と設定**: 退避なら `mv`、消すなら `rm -rf`。パスは 1 で確かめたものだけ
3. **スキルと symlink**: 先に symlink(リンク先が jany のスキルのものだけ)、次にスキル本体。**このスキル自身(jany-teardown)は最後に消す**
4. **バイナリ**: `cargo uninstall jany`。`~/.cargo/bin` 以外にある jany(Homebrew など)は消さずに、場所を報告する

### 5. 確かめる

```
grep -n 'jany' <書き換えた rc>       # 3 で見せた行が無い
ls ~/.config/jany ~/.agents/skills/jany-* 2>&1
command -v jany
```

### 6. 報告

- 外した rc の行(ファイルと中身)と、rc のバックアップの場所
- 退避先、または消したパス
- 消したスキルと symlink
- バイナリを消したか
- 残したもの(選ばなかったもの、迷って触らなかった行)
- 今開いているシェルには jany の関数が残っている。`exec zsh`(bash なら `exec bash`、fish なら `exec fish`)で開き直す
- 退避したなら、戻し方: `mv ~/jany-backup-<YYYYMMDD> ~/.config/jany`
