# bb-bin-monkey
bb-bin-monkey is a nats message sender. It composes a [Frame] type message

```sh
Usage: bb-bin-monkey.exe [OPTIONS] --subject <SUBJECT> --cmd <CMD>

Options:
  -s, --subject <SUBJECT>      Name of the person to greet
  -c, --cmd <CMD>              Number of times to greet
  -a, --args <ARGS>
  -n, --nats-addr <NATS_ADDR>  [default: nats://10.2.4.106:4222]
  -h, --help                   Print help
  -V, --version                Print version
```

### Installation:
#### w/ Cargo:
```sh
cargo install --git ssh://git@github.com/BlueBastion/DEV-bb-lib-bastion.git bb-bin-monkey              
```

#### w/ Docker:
[Docker Image](https://github.com/BlueBastion/DEV-bb-lib-bastion/pkgs/container/bb-monkey/229024812?tag=v0.3.3)  

##### Pull Image:
```sh
docker pull ghcr.io/bluebastion/bb-monkey:v0.3.3
```
##### Use in Dockerfile::
```Dockerfile
FROM ghcr.io/bluebastion/bb-monkey:v0.3.3
```



### Example:
```sh
bb-bin-monkey --subject "gc-api.exec" --cmd "_all"
```
