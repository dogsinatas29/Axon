use std::path::Path;

/// STEP 1: GTK Runtime Tap
/// Generates the physical `LD_PRELOAD` C source code to hook deeply into GTK/GObject.
/// This captures the actual, dirty runtime bytes (pointer lineage, actual callback ordering).
pub struct GtkLdPreloadGenerator;

impl GtkLdPreloadGenerator {
    pub fn generate_hook_source(output_dir: &Path) -> Result<(), String> {
        let c_code = r#"
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
#include <glib-object.h>
#include <gtk/gtk.h>
#include <unistd.h>

// Original function pointers
static void (*original_gtk_widget_destroy)(GtkWidget *widget) = NULL;
static guint (*original_g_idle_add)(GSourceFunc function, gpointer data) = NULL;

// File descriptor for the physical telemetry pipe back to the Axon Daemon
static FILE *telemetry_pipe = NULL;

__attribute__((constructor)) static void init_axon_hooks() {
    original_gtk_widget_destroy = dlsym(RTLD_NEXT, "gtk_widget_destroy");
    original_g_idle_add = dlsym(RTLD_NEXT, "g_idle_add");
    
    // Connect to AXON telemetry pipe
    telemetry_pipe = fopen("/tmp/axon_telemetry.sock", "a");
}

void gtk_widget_destroy(GtkWidget *widget) {
    if (telemetry_pipe) {
        // Capture actual refcount and pointer address before destroy propagation
        fprintf(telemetry_pipe, "EVENT:DESTROY PTR:%p REFCOUNT:%d\n", widget, G_OBJECT(widget)->ref_count);
        fflush(telemetry_pipe);
    }
    if (original_gtk_widget_destroy) {
        original_gtk_widget_destroy(widget);
    }
}

guint g_idle_add(GSourceFunc function, gpointer data) {
    if (telemetry_pipe) {
        // Capture the actual deferred callback enqueuing
        fprintf(telemetry_pipe, "EVENT:IDLE_ADD FUNC:%p DATA:%p\n", function, data);
        fflush(telemetry_pipe);
    }
    return original_g_idle_add ? original_g_idle_add(function, data) : 0;
}

// NOTE: g_signal_emit requires libffi or assembly hooking due to varargs. 
// For this physical tap, we start strictly with destroy and idle queues.
"#;
        std::fs::write(output_dir.join("axon_gtk_tap.c"), c_code).map_err(|e| e.to_string())?;
        Ok(())
    }
}
