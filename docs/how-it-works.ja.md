# 仕組み

打った語が、入力行の 1 行になるまでに何が起きるか。[README に戻る](../README.ja.md)

jany は [jind](https://github.com/yukihirop/jind)(jev × find)と [jurl](https://github.com/yukihirop/jurl)(jev × curl)を 1 本のバイナリに汎化したもの。コマンドごとに **定義ディレクトリ**(ファイル 3 つ、Rust は書かない)があり、コマンドを足すのは新しい crate ではなく LLM スキルの仕事になる。

<p align="center">
  <img src="flow.svg" alt="語 → 規則 → 全部決まった? はい: assemble.sh → プロンプト。いいえ: jev(1 リクエスト)→ repair → assemble.sh → プロンプト。破線の箱はコマンドごとの定義、実線はホスト。" width="880">
</p>

- **規則** — 迷いようのない形は `schema.toml` のデータで決まる: パス、glob、`8080:80`、`KEY=value`、`+7d`、`>10M`、`files` / `delete` / `background` のような表の語。全部の語が決まれば、jany は手元から外に出ない。
- **jev** — 残った語は [jev](https://openrouter.ai)(TypeSafe System One、OpenRouter 経由)に **1 リクエスト** で聞く: 「それぞれの語の役割は何か」と、schema が宣言した追加の質問(単位はどれか、以上か以下か、どの表の語の typo か)。jev は決められた選択肢から選んで確率を返すだけで、コマンドを生成はしない。
- **repair** — 語ごとの jev には見えないことを直す: `days` を `7` に付ける、`except` の後ろの名前を除外に取る、`first_name amanda` を key と value の組にする。
- **assemble.sh** — コマンド自身のスクリプト(stdin JSON → stdout JSON)が、役割の付いた語から `argv` を組む。結果がどれだけ危ないかもここで返す。
- **出力** — コマンド行は stdout、それ以外は stderr。語を解釈できない、または確信度が下限を下回ってコマンドを組めないときは、非ゼロで終わり、代わりに `jany <command> --hint` を理由のコメント付きでプロンプトに置く。`dangerous` な結果(find の `-delete`)は先に read-only で実行してプレビューする。`unsafe` なもの(状態を変えるもの: curl の `POST` / `DELETE`、`docker run`)には 1 行の注意が付き、`autorun` でも入力行に置く。

jev の呼び出しは 1 回 200〜700 ms、$0.0001 未満。

## 弱いところ

- jind と jurl と同じ弱さがある: 裸の語(`log`、`app`)は 2 つの役割のどちらにもなりえて、jev の確信度は 0.6〜0.9 にとどまることが多い。迷いようのない形(`*.log`)で書けば jev を飛ばせる。
- 語彙が閉じないコマンド(任意の SQL、jq のプログラム、ffmpeg のフィルタグラフ)には向かない。よく使う形だけ載せて、残りは素通しする。
- 規則で決まった語も含め、全部の語が文脈として jev に送られる。秘密を含みうる役割(curl のヘッダ、docker の `KEY=value`)は `mask` を宣言し、プレースホルダだけが外に出るようにしている。
