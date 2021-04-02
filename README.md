# terminus_bl
Terminus Boot Loader

## simulator platform 
[terminus](https://github.com/shady831213/terminus)

## main dependencies:
[rutsbi](https://github.com/luojia65/rustsbi)

## usage
```
cd fs
tar -zxvf rootfs.ext4.tar.gz

cd ..
PAYLOAD_BIN_PATH=./payload_bin/vmlinux.bin  cargo xbuild --release --target riscv64imac-unknown-none-elf <--features=panic-full>
terminus target/riscv64imac-unknown-none-elf/release/terminus_bl --image=fs/rootfs.ext4
```
