use tracing_subscriber::Layer;
use tracing::Subscriber;
use std::sync::Arc;
use crate::events::EventBus;
use axon_core::{Event, EventType, EventLevel};
use chrono::Local;
use uuid::Uuid;
use once_cell::sync::OnceCell;

static EVENT_BUS: OnceCell<Arc<EventBus>> = OnceCell::new();

pub struct EventBusLayer;

impl EventBusLayer {
    pub fn init(event_bus: Arc<EventBus>) {
        let _ = EVENT_BUS.set(event_bus);
    }
}

impl<S> Layer<S> for EventBusLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let Some(event_bus) = EVENT_BUS.get() else { return };

        let mut visitor = LogVisitor::default();
        event.record(&mut visitor);

        let metadata = event.metadata();
        let level = match *metadata.level() {
            tracing::Level::ERROR => EventLevel::Error,
            tracing::Level::WARN => EventLevel::Warning,
            tracing::Level::INFO => EventLevel::Info,
            _ => EventLevel::Info,
        };

        // v0.0.29: Signal filtering - exclude sensitive or noisy internal logs from broadcasting
        let target = metadata.target();
        if target.starts_with("axum") || target.starts_with("hyper") || target.starts_with("tower_http") {
            return;
        }

        let content = visitor.message.unwrap_or_else(|| "No message".to_string());
        
        // Critical detection (e.g., Failures, Rejects, or Fatal errors)
        let final_level = if content.contains("❌") || content.contains("ERROR") || content.contains("FATAL") || content.contains("REJECT") || content.contains("failed") {
            EventLevel::Critical
        } else {
            level
        };

        let axon_event = Event {
            id: Uuid::new_v4().to_string(),
            project_id: "system".to_string(),
            thread_id: None,
            agent_id: None,
            event_type: EventType::SystemLog,
            level: final_level,
            source: target.to_string(),
            content,
            payload: None,
            timestamp: Local::now(),
        };

        event_bus.publish(axon_event);
    }
}

#[derive(Default)]
struct LogVisitor {
    message: Option<String>,
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        }
    }
}
