# 例

打つ語と、jany が入力行に置く 1 行。[README に戻る](../README.ja.md)

```sh
$ jany find log files older than 7 days in logs delete
this command is destructive.
$ find logs -type f -iname '*.log' -mtime +7
  logs/old-access.log
  logs/kernel.log
  logs/system.log
  … 2 more

$ find logs -type f -iname '*.log' -mtime +7 -delete█
```

最後の行は出力ではない。**入力済みの次のプロンプト**だ。jany は shell-quote した 1 行を stdout に出し、`jany --init zsh` のラッパーがそれを入力行に置く。読んで、必要なら直して、Enter を押す。

```sh
$ jany curl psot localhsot 3000 users first_name amanda      # typo、裸のポート番号、2 語に分かれた key value
$ curl -sS -X POST http://localhost:3000/users -H 'Content-Type: application/json' -H 'Accept: application/json' --data '{"first_name":"amanda"}'

$ jany docker run nginx 8080:80 background named web
$ docker run -d --name web -p 8080:80 nginx

$ jany tar extrct app.tar.gz into dist strip 1                # /jany-register スキルが書いた定義
$ tar -xf app.tar.gz --strip-components 1 -C dist

$ jany find empty folders depth 2 count
$ find . -maxdepth 2 -type d -empty | wc -l
```
