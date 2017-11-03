Install the requirements:

For compiling assembler and building/running the iso:
```
sudo apt-get install nasm xorriso grub-common qemu-system
```
For cross-compiling the rust, install [rustup](https://www.rustup.rs/):
```
curl https://sh.rustup.rs -sSf | sh
rustup override add nightly
rustup component add rust-src
cargo install xargo
```

And then to run:

```
make run
```

For debugging, setup [gdb](https://www.gnu.org/software/gdb/) like [this](https://os.phil-opp.com/set-up-gdb/)
