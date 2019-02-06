you are running nix/NixOS and have ever encountered the following problem?
```
./factorio
bash: ./factorio: No such file or directory
```
fear not, now there is nix-autobahn which will download necessary dependencies for you.
```
nix-autobahn factorio
```
## Files
* find-libs --> Prints out all required libraries by that binary
* fhs-shell --> Spawns a fhsUserEnv nix shell with packages defined as args
* nix-autobahn --> Combines the two scripts above to have an instant working nix-shell


## Dependencies
- expect
- find

alpha version release
