use crate::types::{Attribute, LogLevel};
use crate::{logger::Logger as VigilantLogger, TracingAdapterBuilder};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::field::Field;
use tracing::level_filters::LevelFilter;
use tracing::span::{Attributes as TracingAttributes, Id};
use tracing::{Event, Subscriber};
use tracing_subscriber::field::Visit;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

#[derive(Clone)]
pub struct TracingAdapter {
    inner: Arc<TracingAdapterInner>,
}

struct TracingAdapterInner {
    vigilant_logger: VigilantLogger,
    level_filter: LevelFilter,
}

impl TracingAdapter {
    pub fn new(vigilant_logger: VigilantLogger, level_filter: LevelFilter) -> Self {
        Self {
            inner: Arc::new(TracingAdapterInner {
                vigilant_logger,
                level_filter,
            }),
        }
    }

    pub fn builder<'a>() -> TracingAdapterBuilder<'a> {
        TracingAdapterBuilder::new()
    }

    pub fn shutdown(&self) -> std::io::Result<()> {
        self.inner.vigilant_logger.shutdown()
    }
}

impl<S> Layer<S> for TracingAdapter
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let logger = &self.inner.vigilant_logger;
        let metadata = event.metadata();

        let level = match *metadata.level() {
            tracing::Level::ERROR => LogLevel::ERROR,
            tracing::Level::WARN => LogLevel::WARNING,
            tracing::Level::INFO => LogLevel::INFO,
            tracing::Level::DEBUG | tracing::Level::TRACE => LogLevel::DEBUG,
        };

        if !self.inner.level_filter.enabled(metadata, ctx.clone()) {
            return;
        }

        let mut fields_map = HashMap::new();
        let mut visitor = AllFieldsVisitor(&mut fields_map);
        event.record(&mut visitor);

        let mut attributes: Vec<Attribute> = fields_map
            .into_iter()
            .map(|(k, v)| Attribute::new(k, v))
            .collect();

        attributes.push(Attribute::new("target", metadata.target().to_string()));
        if let Some(file) = metadata.file() {
            attributes.push(Attribute::new("file", file));
        }
        if let Some(line) = metadata.line() {
            attributes.push(Attribute::new("line", line.to_string()));
        }
        if let Some(module_path) = metadata.module_path() {
            attributes.push(Attribute::new("module_path", module_path));
        }
        if let Some(current_span) = ctx.current_span().id() {
            if event.parent().is_none() {
                attributes.push(Attribute::new(
                    "trace.span.id",
                    format!("{:x}", current_span.into_u64()),
                ));
            }
            if let Some(span) = ctx.span(current_span) {
                attributes.push(Attribute::new(
                    "trace.span.name",
                    span.metadata().name().to_string(),
                ));
            }
            if let Some(parent_id) = event.parent() {
                attributes.push(Attribute::new(
                    "trace.span.parent.id",
                    format!("{:x}", parent_id.into_u64()),
                ));
            }
        }

        let message = attributes
            .iter()
            .find(|attr| attr.key == "message")
            .map(|attr| attr.value.clone())
            .unwrap_or_else(|| "<no message>".to_string());

        match level {
            LogLevel::ERROR => logger.error_with_attrs(&message, attributes),
            LogLevel::WARNING => logger.warn_with_attrs(&message, attributes),
            LogLevel::INFO => logger.info_with_attrs(&message, attributes),
            LogLevel::DEBUG => logger.debug_with_attrs(&message, attributes),
        }
    }

    fn on_new_span(&self, attrs: &TracingAttributes<'_>, id: &Id, _ctx: Context<'_, S>) {
        let logger = &self.inner.vigilant_logger;
        let mut attributes = Vec::new();

        if let Some(parent) = attrs.parent() {
            attributes.push(Attribute::new(
                "trace.span.parent.id",
                format!("{:x}", parent.into_u64()),
            ));
        }

        attributes.push(Attribute::new(
            "trace.span.id",
            format!("{:x}", id.into_u64()),
        ));

        attributes.push(Attribute::new(
            "trace.span.name",
            attrs.metadata().name().to_string(),
        ));

        let span_name = attrs.metadata().name();
        let message = format!("Entered span: {}", span_name);

        logger.info_with_attrs(&message, attributes);
    }
}

struct AllFieldsVisitor<'a>(&'a mut HashMap<String, String>);

impl<'a> Visit for AllFieldsVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.0
            .insert(field.name().to_string(), format!("{:?}", value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }
}
