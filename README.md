# Vleue Sentry Reporter

![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![Doc](https://docs.rs/vleue_sentry/badge.svg)](https://docs.rs/vleue_sentry)
[![Crate](https://img.shields.io/crates/v/vleue_sentry.svg)](https://crates.io/crates/vleue_sentry)
[![Bevy Tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![CI](https://github.com/vleue/vleue_sentry/actions/workflows/ci.yml/badge.svg)](https://github.com/vleue/vleue_sentry/actions/workflows/ci.yml)

Error reporting for Bevy using [Sentry](https://sentry.io).

## Usage

Set the login subscriber:

```rust
use bevy::{prelude::*, log::LogPlugin};

use vleue_sentry::sentry_panic_reporter;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            update_subscriber: Some(sentry_panic_reporter),
            ..default()
        }));
}
```
