clang \
  --target=x86_64-unknown-elf \
  -Wall \
  -Wextra \
  -nostdinc \
  -ffreestanding \
  -fno-stack-protector \
  -O3 \
  -fno-stack-check \
  -fno-PIC \
  -ffunction-sections \
  -fdata-sections \
  -m64 \
  -march=x86-64 \
  -mno-80387 \
  -mno-mmx \
  -mno-sse \
  -mno-sse2 \
  -mno-red-zone \
  -mcmodel=kernel \
  -c kernel.c \
  -o kernel.o

clang \
  --target=x86_64-unknown-elf \
  -Wl,-m,elf_x86_64 \
    -Wl,--build-id=none \
    -nostdlib \
    -static \
    -z max-page-size=0x1000 \
    -Wl,--gc-sections \
    -T linker.ld \
  kernel.o \
  -o kernel.elf
