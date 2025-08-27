# A minimal sample to reproduce issue #459

To test [issue #459](https://github.com/ohkami-rs/ohkami/issues/459), run `cargo run`, and in another terminal:

```sh
timeout -sKILL 0.01 curl localhost:5000
```

As for v0.23.3, this will cause server panic, and may lead to `process didn't exit successfully`.

v0.23.4 fixes the behavior to safely print warnings.
