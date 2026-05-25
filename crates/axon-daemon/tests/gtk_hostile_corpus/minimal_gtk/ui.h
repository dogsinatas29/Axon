#ifndef UI_H
#define UI_H

// D-1: STAGE 1 - Minimal GTK Topology
// Only pure GTK widget creation and signal wiring. No async/threads.

typedef struct {
    void* window; // Simulated GtkWidget*
    void* button; // Simulated GtkWidget*
} AppUi;

AppUi* ui_create();
void ui_destroy(AppUi* ui);

#endif // UI_H
