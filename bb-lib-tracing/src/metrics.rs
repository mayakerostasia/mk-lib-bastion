// Construct MeterProvider for MetricsLayer
// fn init_meter_provider(endpoint: String) -> color_eyre::Result<()> {
// let exporter = opentelemetry_otlp::new_exporter()
//     .tonic()
//     .with_export_config(
//         get_export_config(endpoint, ConfigType::Metrics)
//     )
//     .build_metrics_exporter(
//         Box::new(DefaultAggregationSelector::new()),
//         Box::new(DefaultTemporalitySelector::new()),
//     )
//     .unwrap();

// let reader = PeriodicReader::builder(exporter, runtime::Tokio)
//     .with_interval(std::time::Duration::from_secs(3))
//     .build();

// For debugging in development
// let stdout_reader = PeriodicReader::builder(
//     opentelemetry_stdout::MetricsExporter::default(),
//     runtime::Tokio,
// )
// .build();

// Rename foo metrics to foo_named and drop key_2 attribute
// let view_one = |instrument: &Instrument| -> Option<Stream> {
//     if instrument.name == "one" {
//         Some(
//             Stream::new()
//                 .name("one-one")
//                 .allowed_attribute_keys(resource().into_iter().map(|k| k.0))
//         )
//     } else {
//         None
//     }
// };

// // Set Custom histogram boundaries for baz metrics
// let view_two = |instrument: &Instrument| -> Option<Stream> {
//     if instrument.name == "two" {
//         Some(
//             Stream::new()
//                 .name("two-one")
//                 .aggregation(Aggregation::ExplicitBucketHistogram {
//                     boundaries: vec![0.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0],
//                     record_min_max: true,
//                 })
//                 .allowed_attribute_keys(resource().into_iter().map(|k| k.0))
//         )
//     } else {
//         None
//     }
// };

// let meter_provider = MeterProvider::builder()
//     .with_resource(resource())
//     .with_reader(reader)
//     // .with_reader(stdout_reader)
//     // .with_view(view_one)
//     // .with_view(view_two)
//     .build();

// global::set_meter_provider(meter_provider.clone());

//     Ok(())
// }