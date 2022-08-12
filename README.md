# GReX Frontend Module Monitor and Control

This application provides an interface to the frontend module (FEM)'s monitor
and control port.

## Buildling

As this needs to run on the rasperry pi, it makes sense to cross-compile and
load instead of compiling on device as Rust has notoriously long compile times.

Here, we'll use the excellent [cross](https://github.com/cross-rs/cross) crate
to automate the cross compilation. Make sure to follow their setup guide.

Then just:

```sh
./build_for_pi.sh
```

## Monitoring

As the FEM spits out data via JSON, this app will capture and deserialize that
payload and present it as a source for prometheus.

## Control

TODO!
