#ifndef TEXT_BUFFER_H
#define TEXT_BUFFER_H

#include <windows.h>

// Function to initialize the text buffer
void TextBufferInit(HWND hwnd);

// Function to append text to the buffer
void TextBufferAppend(HWND hwnd, const char* text);

// Function to clear the text buffer
void TextBufferClear(HWND hwnd);

#endif // TEXT_BUFFER_H