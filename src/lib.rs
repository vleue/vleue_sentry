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

use std::{env, path::PathBuf};

#[cfg(all(feature = "bevy", not(feature = "subcrates")))]
#[doc(hidden)]
pub use bevy::{
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

#[cfg(feature = "subcrates")]
#[doc(hidden)]
pub use bevy_app::App;
#[cfg(feature = "subcrates")]
#[doc(hidden)]
pub use bevy_log::{
    tracing_subscriber::{
        layer::{self, SubscriberExt},
        Layer,
    },
    BoxedSubscriber, Level,
};
#[cfg(feature = "subcrates")]
#[doc(hidden)]
pub use bevy_utils::tracing::{Event, Subscriber};

use sentry::ClientInitGuard;

/// A layer that reports events to Sentry
#[doc(hidden)]
pub struct SentryLayer {
    #[allow(dead_code)]
    guard: ClientInitGuard,
    report_only_panic: bool,
}

impl SentryLayer {
    /// Create the layer
    #[doc(hidden)]
    pub fn new(guard: ClientInitGuard, report_only_panic: bool) -> Self {
        Self {
            guard,
            report_only_panic,
        }
    }
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
        let guard = init((
            sentry_dsn,
            ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ));

        env::args().next().and_then(|file| {
            PathBuf::from(file)
                .file_name()
                .and_then(|file| file.to_str())
                .map(|exe| {
                    configure_scope(|scope| {
                        scope.set_tag("executable", dbg!(exe));
                    });
                })
        });

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
        let guard = init((
            sentry_dsn,
            ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ));

        env::args().next().and_then(|file| {
            PathBuf::from(file)
                .file_name()
                .and_then(|file| file.to_str())
                .map(|exe| {
                    configure_scope(|scope| {
                        scope.set_tag("executable", dbg!(exe));
                    });
                })
        });

        Box::new(subscriber.with(SentryLayer {
            guard,
            report_only_panic: false,
        }))
    } else {
        subscriber
    }
}

#[doc(hidden)]
pub use sentry::{configure_scope, init, ClientOptions};

/// Reports panics and errors to Sentry
/// logs will be added as breadcrumbs
///
/// Unlike the functions, this macro will capture the crate name and version, so it should be used from the main binary crate
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy::log::LogPlugin;
/// use vleue_sentry::sentry_reporter;
///
/// App::new()
///     .add_plugins(DefaultPlugins.set(LogPlugin {
///         update_subscriber: Some(sentry_reporter!(true)),
///         ..default()
///     }));
/// ```
#[macro_export]
macro_rules! sentry_reporter {
    ($report_only_panic:literal) => {
        |_app: &mut vleue_sentry::App, subscriber: vleue_sentry::BoxedSubscriber| {
            if let Ok(sentry_dsn) = std::env::var("SENTRY_DSN") {
                let guard = vleue_sentry::init((
                    sentry_dsn,
                    vleue_sentry::ClientOptions {
                        release: Some(
                            format!(
                                "{}@{}",
                                std::env!("CARGO_CRATE_NAME"),
                                std::env!("CARGO_PKG_VERSION")
                            )
                            .into(),
                        ),
                        ..Default::default()
                    },
                ));

                std::env::args().next().and_then(|file| {
                    std::path::PathBuf::from(file)
                        .file_name()
                        .and_then(|file| file.to_str())
                        .map(|exe| {
                            vleue_sentry::configure_scope(|scope| {
                                scope.set_tag("executable", exe);
                            });
                        })
                });

                Box::new(vleue_sentry::SubscriberExt::with(
                    subscriber,
                    vleue_sentry::SentryLayer::new(guard, $report_only_panic),
                ))
            } else {
                subscriber
            }
        }
    };
}
