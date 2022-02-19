ROOT=./build/root
VMLINUX=./kernel/bzImage
INITRD=./kernel/initrd.cpio.gz

# Standard linux directories
mkdir -p $ROOT/{bin,dev,etc/rc,home,mnt,proc,root,sys,tmp/run,usr/{bin,sbin,lib},var}

# We need a passwd file
cat > "$ROOT"/etc/passwd << 'EOF' &&
root:x:0:0:root:/root:/bin/sh
EOF

# Build user-space
cargo build --release --target x86_64-unknown-linux-musl --bins
cp target/x86_64-unknown-linux-musl/release/{sh,ls,cat} "$ROOT"/bin/
#need to make a proper init at some point the shell will do for now
cp target/x86_64-unknown-linux-musl/release/sh "$ROOT"/init
chmod a+x "$ROOT"/bin/*

# Make the image
(cd $ROOT && find . | cpio -o -H newc | gzip) > $INITRD

# Let's Go!!!!
qemu-system-x86_64 -D qemu.log -nographic -no-reboot -m 256 -kernel $VMLINUX  -initrd $INITRD -append "panic=1 console=ttyS0 init=/bin/sh"