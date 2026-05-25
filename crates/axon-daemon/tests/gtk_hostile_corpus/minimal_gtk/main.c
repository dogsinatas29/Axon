#include "ui.h"
#include <stdio.h>

// STAGE 1: Minimal GTK Topology - Main Entry

void on_button_clicked(void* widget, void* data) {
    printf("Button clicked\n");
}

int main(int argc, char** argv) {
    // Simulate gtk_init
    printf("GTK Init...\n");

    AppUi* ui = ui_create();

    // Simulate g_signal_connect
    // STAGE 2 Strike Target: Malicious AI wiring signals to wrong widgets
    printf("Signal wired...\n");

    // Simulate gtk_main
    printf("GTK Main Loop...\n");

    // Simulate cleanup
    ui_destroy(ui);

    return 0;
}
