use opentelemetry::sdk::trace::{self, Builder, Sampler};
use opentelemetry::sdk::{self, Resource};
use opentelemetry::trace::{TraceError, TracerProvider};
use opentelemetry::KeyValue;
use opentelemetry_otlp::{SpanExporter, SpanExporterBuilder, WithExportConfig};

fn exporter_builder(endpoint: &str) -> SpanExporter {
  SpanExporterBuilder::from(opentelemetry_otlp::new_exporter().tonic().with_endpoint(endpoint))
    .build_span_exporter()
    .unwrap()
}

fn trace_config() -> Option<trace::Config> {
  Some(
    trace::config()
      .with_sampler(Sampler::AlwaysOn)
      .with_resource(Resource::new(vec![KeyValue::new("service.name", "wick")])),
  )
}

pub(super) fn build_batch(endpoint: &str) -> Result<(sdk::trace::Tracer, sdk::trace::TracerProvider), TraceError> {
  Ok(build_with_exporter(
    sdk::trace::TracerProvider::builder()
      .with_batch_exporter(exporter_builder(endpoint), opentelemetry::runtime::Tokio),
    trace_config(),
  ))
}

pub(super) fn build_simple(endpoint: &str) -> Result<(sdk::trace::Tracer, sdk::trace::TracerProvider), TraceError> {
  Ok(build_with_exporter(
    sdk::trace::TracerProvider::builder().with_simple_exporter(exporter_builder(endpoint)),
    trace_config(),
  ))
}

pub(super) fn build_with_exporter(
  mut provider_builder: Builder,
  trace_config: Option<sdk::trace::Config>,
) -> (sdk::trace::Tracer, sdk::trace::TracerProvider) {
  if let Some(config) = trace_config {
    provider_builder = provider_builder.with_config(config);
  }
  let provider = provider_builder.build();
  let tracer = provider.versioned_tracer("opentelemetry-otlp", Some(env!("CARGO_PKG_VERSION")), None);
  (tracer, provider)
}
