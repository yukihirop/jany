# Examples

What you type, and the line jany puts on your prompt. [Back to the README](../README.md)

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

The last line is not output. It is your **next prompt, already filled in**. jany prints one shell-quoted line on stdout, and the wrapper from `jany --init zsh` puts it on the command line. Read it, edit it if you like, press Enter.

```sh
$ jany curl psot localhsot 3000 users first_name amanda      # typos, a bare port, key value as two words
$ curl -sS -X POST http://localhost:3000/users -H 'Content-Type: application/json' -H 'Accept: application/json' --data '{"first_name":"amanda"}'

$ jany docker run nginx 8080:80 background named web
$ docker run -d --name web -p 8080:80 nginx

$ jany tar extrct app.tar.gz into dist strip 1                # a definition written by the /jany-register skill
$ tar -xf app.tar.gz --strip-components 1 -C dist

$ jany find empty folders depth 2 count
$ find . -maxdepth 2 -type d -empty | wc -l
```
