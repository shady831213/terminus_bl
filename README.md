# terminus_bl
Terminus Boot Loader
```
cd fs
tar -zxvf rootfs.ext4.tar.gz

cd ..
PAYLOAD_BIN_PATH=./payload_bin/vmlinux.bin  cargo xbuild --release --target riscv64imac-unknown-none-elf <--features=panic-full>
//terminus refer to https://github.com/shady831213/terminus
terminus target/riscv64imac-unknown-none-elf/release/terminus_bl --image=fs/rootfs.ext4
```
