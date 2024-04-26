#![warn(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    missing_docs
)]
#![doc = include_str!("../README.md")]

use std::env;

use bevy::{
    app::App,
    log::{
        tracing_subscriber::{
            layer::{self, SubscriberExt},
            Layer,
        },
        BoxedSubscriber,
    },
    utils::tracing::{Event, Level, Subscriber},
};

use sentry::ClientInitGuard;

struct SentryLayer {
    #[allow(dead_code)]
    guard: ClientInitGuard,
    report_only_panic: bool,
}

impl<S: Subscriber> Layer<S> for SentryLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: layer::Context<'_, S>) {
        let breadcrumb = sentry_tracing::breadcrumb_from_event(event);
        if event.metadata().level() == &Level::ERROR && !self.report_only_panic {
            sentry::capture_event(sentry::protocol::Event {
                level: breadcrumb.level,
                message: breadcrumb.message,
                timestamp: breadcrumb.timestamp,
                ..Default::default()
            });
        } else {
            sentry::add_breadcrumb(breadcrumb);
        }
    }
}

/// Reports panics to Sentry
/// logs will be added as breadcrumbs
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy::log::LogPlugin;
/// use vleue_sentry::sentry_panic_reporter;
///
/// App::new()
///     .add_plugins(DefaultPlugins.set(LogPlugin {
///         update_subscriber: Some(sentry_panic_reporter),
///         ..default()
///     }));
/// ```
pub fn sentry_panic_reporter(_: &mut App, subscriber: BoxedSubscriber) -> BoxedSubscriber {
    if let Ok(sentry_dsn) = env::var("SENTRY_DSN") {
        let guard = sentry::init((
            sentry_dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ));

        Box::new(subscriber.with(SentryLayer {
            guard,
            report_only_panic: true,
        }))
    } else {
        subscriber
    }
}

/// Reports panics and errors to Sentry
/// logs will be added as breadcrumbs
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy::log::LogPlugin;
/// use vleue_sentry::sentry_error_reporter;
///
/// App::new()
///     .add_plugins(DefaultPlugins.set(LogPlugin {
///         update_subscriber: Some(sentry_error_reporter),
///         ..default()
///     }));
/// ```
pub fn sentry_error_reporter(_: &mut App, subscriber: BoxedSubscriber) -> BoxedSubscriber {
    if let Ok(sentry_dsn) = env::var("SENTRY_DSN") {
        let guard = sentry::init((
            sentry_dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ));

        Box::new(subscriber.with(SentryLayer {
            guard,
            report_only_panic: false,
        }))
    } else {
        subscriber
    }
}
