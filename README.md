you are running nix/NixOS and have ever encountered the following problem?
```
./factorio
bash: ./factorio: No such file or directory
```
fear not, now there is nix-autobahn which will download necessary dependencies for you.
```
./nix-autobahn factorio --> Spawns a fhs shell with all needed shared dependencies to execute the binary
./nix-autobahn-ld factorio -> Spawns you inside a shell with NIX_LD_LIBRARY_PATH set to the needed dependencies
./find-libs factorio    --> Lists all libs needed for the binary
./fhs-shell             --> Spawns a fhs shell. Arguments are the packages to be available
```

## Technical Description
This simple shell script collection allows you to download ELF binaries and use them right away!
This is achieved by enumerating the shared library dependencies from the ELF header and then searching for
the equivalent library in nixpkgs. This is done by querying `nix-locate` locally. To be able to use nix-locate, first,
the index has to be build this is done by running `nix-index` and waiting 10-15 minutes.


## Files
* find-libs --> Prints out all required libraries by that binary
* fhs-shell --> Spawns a fhsUserEnv nix shell with packages defined as args
* nix-autobahn --> Combines the two scripts above to have an instant working nix-shell

## Dependencies
- find
- fzf
- nix-index
- nix-ld (optional) https://github.com/Mic92/nix-ld
