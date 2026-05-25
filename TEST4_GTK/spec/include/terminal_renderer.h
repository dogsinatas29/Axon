===AXON_PATCH_START===
{
  "response": "```cpp
#ifndef TERMINAL_RENDERER_H
#define TERMINAL_RENDERER_H

#include <stddef.h>

void draw_text(double x, double y, const char* text);
const char* get_buffer_line(size_t n);
t size_t get_buffer_line_count();
double get_canvas_height();
double get_canvas_width();
double get_line_height();
void request_redraw();
void set_color(double r, double g, double b);
void set_font_size(double size);

#endif // TERMINAL_RENDERER_H
}
===AXON_PATCH_END===