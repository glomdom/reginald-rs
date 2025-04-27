#define SERIAL_PORT 0x3F8

static inline void outb(unsigned short port, unsigned char data) {
  asm volatile("outb %0, %1" : : "a"(data), "Nd"(port));
}

static inline unsigned char inb(unsigned short port) {
  unsigned char ret;
  asm volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
  return ret;
}

void serial_init(void) {
  outb(SERIAL_PORT + 1, 0x00); // Disable interrupts
  outb(SERIAL_PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
  outb(SERIAL_PORT + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
  outb(SERIAL_PORT + 1, 0x00); //                  (hi byte)
  outb(SERIAL_PORT + 3, 0x03); // 8 bits, no parity, one stop bit
  outb(SERIAL_PORT + 2,
       0xC7); // Enable FIFO, clear them, with 14-byte threshold
  outb(SERIAL_PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
}

// Check if transmit FIFO is empty
int serial_is_transmit_fifo_empty() { return inb(SERIAL_PORT + 5) & 0x20; }

// Write a character
void serial_write_char(char c) {
  while (!serial_is_transmit_fifo_empty())
    ; // Wait until FIFO is ready
  outb(SERIAL_PORT, c);
}

// Write a string
void serial_write_string(const char *str) {
  while (*str) {
    if (*str == '\n')
      serial_write_char('\r'); // Proper CRLF for terminals
    serial_write_char(*str++);
  }
}

typedef struct {
  unsigned char *buffer_addr;
  long long size;
  long long stride;
  long long width;
  long long height;
  unsigned int pixel_format;
} FramebufferInfo;

void put_pixel(FramebufferInfo* fb, int x, int y, unsigned char r, unsigned char g, unsigned char b) {
  unsigned int *pixel = (unsigned int*)(fb->buffer_addr + (y * fb->stride + x) * 4);

  *pixel = (b) | (g << 8) | (r << 16);
}

void draw_rect(FramebufferInfo* fb, int x, int y, int w, int h, unsigned char r, unsigned char g, unsigned char b) {
  unsigned int color = (b) | (g << 8) | (r << 16);
  unsigned int *pixels = (unsigned int*)fb->buffer_addr;

  for (int dy = 0; dy < h; dy++) {
      int py = y + dy;
      if (py >= fb->height) break; // Prevent drawing off-screen

      for (int dx = 0; dx < w; dx++) {
          int px = x + dx;
          if (px >= fb->width) break; // Prevent drawing off-screen

          pixels[py * fb->stride + px] = color;
      }
  }

  serial_write_string("drawed\n");
}

void _start(FramebufferInfo *fb) {
  serial_init();
  serial_write_string("hello qemu serial\n");

  for (unsigned int i = 0; i < 5000; i++) {
      fb->buffer_addr[i] = 0xFF;
  }


  draw_rect(fb, 0, 0, 40, 40, 255, 255, 255);

  serial_write_string("we cleared\n");

  while (1) {
    __asm__ volatile("hlt");
  }
}
