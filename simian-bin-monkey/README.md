# simian-bin-monkey
simian-bin-monkey is a nats message sender. It composes a [Frame] type message

```sh
Usage: simian-bin-monkey.exe [OPTIONS] --subject <SUBJECT> --cmd <CMD>

Options:
  -s, --subject <SUBJECT>      Name of the person to greet
  -c, --cmd <CMD>              Number of times to greet
  -a, --args <ARGS>
  -n, --nats-addr <NATS_ADDR>  [default: nats://10.2.4.106:4222]
  -h, --help                   Print help
  -V, --version                Print version
```

### Example:
```sh
simian-bin-monkey --subject "gc-api.exec" --cmd "_all"
```