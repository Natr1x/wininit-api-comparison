# New Trait based wininit Api vs the Old

For some reason I cannot get the new api to handle resizing without
flickering when using wgpu and wasm.

The old api also behaves a bit weirdly in this example.
But I haven't checked if that also happens in isolation.

## Building

The actual examples need to be built with [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).

When you have that installed you can run the following commands:

```sh
wasm-pack build --dev --target web web/old-api
wasm-pack build --dev --target web web/new-api
```
Which should put the files in the correct place.


## Running

The root cargo project in the repo is just a very basic webserver for which can
serve the example wasm code.

You can build and run it by simply running `cargo run` from the root of this repo.

If you have [built](#building) the examples then it should serve them.

