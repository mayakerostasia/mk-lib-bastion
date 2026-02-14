// use std::str::FromStr;

// use sentry::{self, ClientInitGuard};
// use sentry::types::Dsn;

// #[allow(unused)]
// pub fn init_sentry_tracer() -> ClientInitGuard {
//     let _sentry_guard = sentry::init(sentry::ClientOptions {
//         dsn: Some(
//                 Dsn::from_str(std::env::var("SENTRY_DSN")
//                     .expect("SENTRY_DSN must be set")
//                     .as_str()
//                 ).unwrap()
//         ),
//         release: Some(
//             std::env::var("GITHUB_SHA")
//                 .expect("GITHUB_SHA must be set")
//                 .into(),
//         ),
//         environment: Some(
//             std::env::var("ENV")
//                 .expect("ENV must be set")
//                 .into(),
//         ),
//         ..Default::default()
//     });
//     _sentry_guard
// }
