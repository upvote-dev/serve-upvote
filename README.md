serve-upvote
============
[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Server entrypoint for auth, `Review`s, `Profile`s, and more.

Backend implementation to be found at child repository: https://github.com/upvote-dev/upvote-backend (clone this one
directory above to build)

## Why

Honestly everything else is too complicated. When you just want some access tokens and to get started real quick, this
is what you need.

Once the server is up—setup/usage notes follow—you can just:

```sh
$ curl -X POST http://localhost:3000/api/token \
       -H 'Content-Type: application/json' \
       -H 'Accept: application/json' \
       -d '{"grant_type": "password", "username": "user", "password": "pass"}'
```

Which—registering user if nonexistent—returns of the form:

```json
{
  "access_token": "user::regular::access_token::4151d642-eb27-4064-b87c-e3d2bfa10435",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

Then you can use this the regular way, for example:

```sh
$ curl http://localhost:3000/secured/secret \
       -H 'Authorization: Bearer user::regular::access_token::4151d642-eb27-4064-b87c-e3d2bfa10435'
```

See [src/main.rs](src/main.rs) for current routes.
OpenAPI docs are available at http://localhost:3000/rapidoc and http://localhost:3000/redoc.
Also in [openapi.yml](openapi.yml) and [openapi.md](openapi.md).

## Docker usage

Install Docker, and then run the following, which will make a server available at http://localhost:3000:

```sh
$ docker compose up
````

NOTE: You may need to configure this for your architecture first, for example:

```sh
$ docker compose build --build-arg ARCH_VARIANT='amd64' \
                       --build-arg ARCH='x86_64'
$ docker compose up
```

Or to work with just one image and provide your own database and redis:

```sh
$ docker build -f 'debian.Dockerfile' -t "${PWD##*/}"':latest' .
$ docker run -e DATABASE_URL="$RDBMS_URI" \
             -e REDIS_URL='localhost:6379' \
             -p '3000:3000' \
             --name 'serve_api' \
             "${PWD##*/}"
```

## Reverse proxy

### Nginx

Add this to your service block:

    location ~* /(api|redoc|rapidoc) {
        proxy_pass http://localhost:3000;
    }

## Native usage

Install Rust, `git`, and ensure you have your PostgreSQL and Redis/Valkey services setup.

### Environment setup

Add an `.env` file or otherwise add these environment variables; replacing connection strings with what you use:

    DATABASE_URL=postgres://rest_user:rest_pass@localhost/rest_db
    REDIS_URL=redis://127.0.0.1/

### Deployment

    cargo build --release

Take note of the `target/release/serve-upvote` location, then use it:

#### Systemd

On most Linux operating systems systemd is the default init system, here are the config files you'll need:

##### Valkey

    [Unit]
    Description=Valkey
    After=network-online.target
    
    [Service]
    ExecStart=/usr/local/bin/valkey-server /etc/valkey.conf
    Restart=always
    
    [Install]
    WantedBy=multi-user.target

With this `valkey.conf`:

    bind 127.0.0.1 ::1
    port 6379
    timeout 0
    tcp-keepalive 300
    pidfile /run/valkey.pid
    loglevel notice
    logfile ""
    databases 16
    always-show-logo no
    set-proc-title yes
    proc-title-template "{title} {listen-addr} {server-mode}"
    locale-collate ""
    stop-writes-on-bgsave-error yes
    rdbcompression yes
    rdbchecksum yes
    dbfilename dump.rdb
    rdb-del-sync-files no
    dir /var/db/valkey/
    replica-serve-stale-data yes
    replica-read-only yes
    repl-diskless-sync yes
    repl-diskless-sync-delay 5
    repl-diskless-sync-max-replicas 0
    repl-diskless-load disabled
    dual-channel-replication-enabled no
    repl-disable-tcp-nodelay no
    replica-priority 100
    acllog-max-len 128
    lazyfree-lazy-eviction yes
    lazyfree-lazy-expire yes
    lazyfree-lazy-server-del yes
    replica-lazy-flush yes
    lazyfree-lazy-user-del yes
    lazyfree-lazy-user-flush yes
    oom-score-adj no
    oom-score-adj-values 0 200 800
    disable-thp yes
    appendonly no
    appendfilename "appendonly.aof"
    appenddirname "appendonlydir"
    appendfsync everysec
    no-appendfsync-on-rewrite no
    auto-aof-rewrite-percentage 100
    auto-aof-rewrite-min-size 64mb
    aof-load-truncated yes
    aof-use-rdb-preamble yes
    aof-timestamp-enabled no
    slowlog-log-slower-than 10000
    slowlog-max-len 128
    latency-monitor-threshold 0
    notify-keyspace-events "
    hash-max-listpack-entries 512
    hash-max-listpack-value 64
    list-max-listpack-size -2
    list-compress-depth 0
    set-max-intset-entries 512
    set-max-listpack-entries 128
    set-max-listpack-value 64
    zset-max-listpack-entries 128
    zset-max-listpack-value 64
    hll-sparse-max-bytes 3000
    stream-node-max-bytes 4096
    stream-node-max-entries 100
    activerehashing yes
    client-output-buffer-limit normal 0 0 0
    client-output-buffer-limit replica 256mb 64mb 60
    client-output-buffer-limit pubsub 32mb 8mb 60
    hz 10
    dynamic-hz yes
    aof-rewrite-incremental-fsync yes
    rdb-save-incremental-fsync yes
    jemalloc-bg-thread yes

##### `/etc/systemd/system/serve-upvote.service`

    [Unit]
    Description=serve-upvote
    After=network-online.target
    
    [Service]
    Environment="DATABASE_URL=postgres://rest_user:rest_pass@localhost/rest_db"
    ExecStart=/mylocation/serve-upvote/target/release/serve-upvote
    Restart=always
    
    [Install]
    WantedBy=multi-user.target 

##### `/lib/systemd/system/postgresql@.service`

Provided by PostgreSQL package. I just use the default.

### Execute locally, without system init system

    cargo run

#### `--help`

    Usage: serve-upvote [OPTIONS]
    
    Options:
          --hostname <HOSTNAME>  Hostname [env: SADAS_HOSTNAME=] [default: localhost]
      -p, --port <PORT>          Port [env: SADAS_PORT=] [default: 3000]
          --no-host-env          Avoid inheriting host environment variables
          --env-file <ENV_FILE>  Env file, defaults to ".env"
      -e, --env <ENV>            Env var (can be specified multiple times, like `-eFOO=5 -eBAR=can`)
      -h, --help                 Print help
      -V, --version              Print version

## Contribution guide

Ensure all tests are passing [`cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html) and [
`rustfmt`](https://github.com/rust-lang/rustfmt) has been run. This can be with [
`cargo make`](https://github.com/sagiegurari/cargo-make); installable with:

```sh
$ cargo install --force cargo-make
```

Then run:

```sh
$ cargo make
```

Finally, we recommend [feature-branches](https://martinfowler.com/bliki/FeatureBranch.html) with an
accompanying [pull-request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/about-pull-requests).
</small>

<hr/>

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.
