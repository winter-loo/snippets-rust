pub use sentry::release_name;
use std::borrow::Cow;
use std::env;
use sentry::ClientInitGuard;

#[must_use]
pub fn init_sentry(
    release_name: Option<Cow<'static, str>>,
    extra_options: &[(&str, &str)],
) -> Option<ClientInitGuard> {
    // get from https://sentry.io
    let dsn = env::var("SENTRY_DSN").expect("the environment variable SENTRY_DSN need be set");
    let environment = env::var("SENTRY_ENVIRONMENT").unwrap_or_else(|_| "development".into());

    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: release_name,
            environment: Some(environment.into()),
            ..Default::default()
        },
    ));
    sentry::configure_scope(|scope| {
        for &(key, value) in extra_options {
            scope.set_extra(key, value.into());
        }
    });
    Some(guard)
}

fn main() {
    // initialize sentry if SENTRY_DSN is provided
    let _sentry_guard = init_sentry(
        Some("initial".into()),
        &[("node_id", "123")],
    );

    // Sentry will capture this
    panic!("Everything is on fire!");
}

