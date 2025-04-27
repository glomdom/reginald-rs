rm -rf esp/
mkdir -p esp/efi/boot/

cd test-kernel/
./build.sh
cd ..

mv -v test-kernel/kernel.elf esp/efi/boot/

cargo b || exit
cp target/x86_64-unknown-uefi/debug/reginald-rs.efi esp/efi/boot/bootx64.efi

qemu-system-x86_64 -enable-kvm \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:esp \
    -serial stdio