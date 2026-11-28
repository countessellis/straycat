![Stray Cat](/resources/straycat-small.png)

---
---

# Stray Cat

An network and protocol troubleshooting utility in the spirit of netcat but with wider application.

Currently, only text based interaction with a websocket is implemented. 

## Reference

* [Changelog](CHANGELOG.md)
* [To Do List](TODO.md)

## Usage

```
Usage: straycat [(OPTIONS)]

  An network and protocol troubleshooting utility in the spirit of netcat but with wider application.

Options:

  -C                                   Send CRLF as line-ending.
  -p <PORT>                            Specify the source port, 0-65536.
  -s <ADDR>                            Specify the source IP, in IPv4 or IPv6.

  --mode <MODE>                        Mode to run in (default: batch, choices: batch, interactive, help)
  --target <MODE>                      Target to connect to, either a URL, a hostname, or a hostname:port pair.
  --port <MODE>                        Target port to connect to.
  --secure <true/false>                Use TLS to connect (default: false)
  --pretty <MODE>                      Enable pretty print of JSON coming from target (default: false)

  --verbose <true/false>               Enable or disable logging (default: false)
  --debug <true/false>                 Enable or disable debug level logging (default: false)
  --help | -h                          Display this help message
  --version | -V                       Display version
```

---
---

©2026 Bethany Davis <ellis@despoena.com>.

