# Overview

You are running [Nix](https://nixos.org/nixos/)/[NixOS](https://nixos.org/nix/)
and have ever encountered the following problem? 

```bash
> ./factorio
bash: ./factorio: No such file or directory
```
Fear not, now there is `nix-autobahn` which will help you running those
binaries:

```bash
> nix-autobahn factorio
Pick provider for libX11.so.6: xlibs.libX11.out /lib/libX11.so.6
Pick provider for libXinerama.so.1: xlibs.libXinerama.out /lib/libXinerama.so.1
Pick provider for libpulse-simple.so.0: pulseaudioFull.out /lib/libpulse-simple.so.0
```

`nix-autobahn` looks through the output of `ldd factorio` for missing shared
objects (`*.so*` files). For each missing one nix-index's `nix-locate` is used
to determine provider candidates (read: packages that offer this shared
object). If more than one candidate is found a nice™<sup>™</sup>
[TUI](https://docs.rs/dialoguer/) asks you to pick one. All of the chosen
packages are baked into a [FHS-compatible
Sandbox](https://nixos.org/nixpkgs/manual/#sec-fhs-environments) that can be
built through a nix expression inside of a shell script.

All that is left from now is to run it:
```bash
> ./run-with-nix
# ...
   0.010 Error SDLWindow.cpp:186: Failed to create an application window. SDL_Error: Failed loading libGL.so.1: libGL.so.1: cannot open shared object file: No such file or directory
   0.010 Error Util.cpp:83: Failed to create an application window. SDL_Error: Failed loading libGL.so.1: libGL.so.1: cannot open shared object file: No such file or directory
   1.316 Goodbye
```

Sadly, that didn't work so well. Fortunately we know why: We need a provider
for `libGL.so.1`. Now we have two options: Either we pass `-l libGL.so.1` to
`nix-autobahn` to tell it to find a provider for that given shared object or we
just know the missing package is `libGL`. In the latter case we can pass `-p
libGL` to `nix-autobahn` to ensure that a provider for `libGL` ends up in the
build expression:

```bash
> nix-autobahn factorio -p libGL
Pick provider for libX11.so.6: xlibs.libX11.out /lib/libX11.so.6
Pick provider for libXinerama.so.1: xlibs.libXinerama.out /lib/libXinerama.so.1
Pick provider for libpulse-simple.so.0: pulseaudioFull.out /lib/libpulse-simple.so.0
> ./run-with-nix
```

And we are good to go!

# Usage

```bash
USAGE:
    nix-autobahn [OPTIONS] <BINARY>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --additional-libs <libs>...        Additional libraries to search for and propagate
    -p, --additional-pkgs <packages>...    Additional packages to propagate

ARGS:
    <BINARY>    dynamically linked binary to be examined
```

# Dependencies

- `nix-index`. Ensure that both `nix-index` is installed __and__ has a valid
  index. To rebuild the index simply run `nix-index`. For further information
  refer to `nix-index`'s manual: `nix-index --help`. Rebuilding the index is
  not frequently needed, in fact we recommend doing so if you ran into problems
  or changed your channel (e.g. `nixos-19.03` -> `nixos-19.09`).

