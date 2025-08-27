# WebSocket Example

## Feature flags description

- `DEBUG`: Enables Ohkami's debug logging.
- `tls`: Enables TLS support ( https://, wss:// ).

## Prerequisites

If you want to run this example with TLS support, you need to have
[`mkcert`](https://github.com/FiloSottile/mkcert) and run:

```sh
# assuming you have mkcert installed and `mkcert -install` has already executed:
mkcert -key-file key.pem -cert-file cert.pem localhost 127.0.0.1
```
