#include "ui.h"
#include <stdlib.h>
#include <stdio.h>

AppUi* ui_create() {
    AppUi* ui = malloc(sizeof(AppUi));
    // Simulate gtk_window_new
    ui->window = malloc(1); 
    // Simulate gtk_button_new
    ui->button = malloc(1); 
    return ui;
}

void ui_destroy(AppUi* ui) {
    if (ui) {
        // Simulate gtk_widget_destroy
        free(ui->button);
        free(ui->window);
        free(ui);
    }
}
