// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Unitycatalog-RS Authors

#![deny(clippy::all)]

use std::collections::HashMap;

use env_logger::Env;
use napi_derive::*;

mod client;
mod error;

/// Timeout configuration for remote HTTP client.
#[napi(object)]
#[derive(Debug)]
pub struct TimeoutConfig {
    /// The timeout for establishing a connection in seconds. Default is 120
    /// seconds (2 minutes). This can also be set via the environment variable
    /// `UC_CONNECT_TIMEOUT`, as an integer number of seconds.
    pub connect_timeout: Option<f64>,
    /// The timeout for reading data from the server in seconds. Default is 300
    /// seconds (5 minutes). This can also be set via the environment variable
    /// `UC_READ_TIMEOUT`, as an integer number of seconds.
    pub read_timeout: Option<f64>,
    /// The timeout for keeping idle connections in the connection pool in seconds.
    /// Default is 300 seconds (5 minutes). This can also be set via the
    /// environment variable `UC_CONNECTION_TIMEOUT`, as an integer
    /// number of seconds.
    pub pool_idle_timeout: Option<f64>,
}

/// Retry configuration for the remote HTTP client.
#[napi(object)]
#[derive(Debug)]
pub struct RetryConfig {
    /// The maximum number of retries for a request. Default is 3. You can also
    /// set this via the environment variable `UC_MAX_RETRIES`.
    pub retries: Option<u8>,
    /// The maximum number of retries for connection errors. Default is 3. You
    /// can also set this via the environment variable `UC_CONNECT_RETRIES`.
    pub connect_retries: Option<u8>,
    /// The maximum number of retries for read errors. Default is 3. You can also
    /// set this via the environment variable `UC_READ_RETRIES`.
    pub read_retries: Option<u8>,
    /// The backoff factor to apply between retries. Default is 0.25. Between each retry
    /// the client will wait for the amount of seconds:
    /// `{backoff factor} * (2 ** ({number of previous retries}))`. So for the default
    /// of 0.25, the first retry will wait 0.25 seconds, the second retry will wait 0.5
    /// seconds, the third retry will wait 1 second, etc.
    ///
    /// You can also set this via the environment variable
    /// `UC_RETRY_BACKOFF_FACTOR`.
    pub backoff_factor: Option<f64>,
    /// The jitter to apply to the backoff factor, in seconds. Default is 0.25.
    ///
    /// A random value between 0 and `backoff_jitter` will be added to the backoff
    /// factor in seconds. So for the default of 0.25 seconds, between 0 and 250
    /// milliseconds will be added to the sleep between each retry.
    ///
    /// You can also set this via the environment variable
    /// `UC_RETRY_BACKOFF_JITTER`.
    pub backoff_jitter: Option<f64>,
    /// The HTTP status codes for which to retry the request. Default is
    /// [429, 500, 502, 503].
    ///
    /// You can also set this via the environment variable
    /// `UC_RETRY_STATUSES`. Use a comma-separated list of integers.
    pub statuses: Option<Vec<u16>>,
}

#[napi(object)]
#[derive(Debug, Default)]
pub struct ClientConfig {
    pub user_agent: Option<String>,
    pub retry_config: Option<RetryConfig>,
    pub timeout_config: Option<TimeoutConfig>,
    pub extra_headers: Option<HashMap<String, String>>,
}

#[napi::module_init]
fn init() {
    let env = Env::new()
        .filter_or("UC_LOG", "warn")
        .write_style("UC_LOG_STYLE");
    env_logger::init_from_env(env);
}
