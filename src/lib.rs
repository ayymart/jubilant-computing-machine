#[cfg(test)]
mod tests {
    use opentelemetry::trace::{Tracer as _, TracerProvider as _};
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt;

    // This is what I want to use
    #[tokio::test(flavor = "multi_thread")]
    async fn bad_tracing_global() {
        let provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .unwrap();
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("my_tracer")))
            .init();
        opentelemetry::global::set_tracer_provider(provider);
        
        // Tracing API exports to stdout, but not otlp
        do_tracing();

        // Otel API doesn't export anything
        do_otel();

        opentelemetry::global::shutdown_tracer_provider();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn good_tracing_threadlocal() {
        let provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .unwrap();
        let subscriber = tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("my_tracing_tracer")));
        let _guard = subscriber.set_default();
        opentelemetry::global::set_tracer_provider(provider);

        // Tracing API exports both to stdout and otlp
        do_tracing();

        // Opentelemetry API also exports
        do_otel();
        
        opentelemetry::global::shutdown_tracer_provider();
    }

    fn do_tracing() {
        let span = tracing::trace_span!("my_tracing_span");
        let _enter = span.enter();
        tracing::trace!("my_tracing_event");
    }

    fn do_otel() {
        let tracer = opentelemetry::global::tracer("my_otel_tracer");
        tracer.in_span("my_otel_span", |_| {
            opentelemetry::trace::get_active_span(|span| {
                span.add_event("my_otel_event", vec![]);
            })
        });
    }
}
