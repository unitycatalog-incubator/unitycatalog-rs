// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The UnityCatalog-RS Authors

pub type Result<T> = napi::Result<T>;

pub trait NapiErrorExt<T> {
    /// Convert to a napi error using from_reason(err.to_string())
    fn default_error(self) -> Result<T>;
}

impl<T> NapiErrorExt<T> for std::result::Result<T, unitycatalog_client::Error> {
    fn default_error(self) -> Result<T> {
        self.map_err(|err| convert_error(&err))
    }
}

pub fn convert_error(err: &unitycatalog_client::Error) -> napi::Error {
    // Emit a structured prefix for typed TS errors so the generated
    // `parseNativeError` function can match on the error code.
    if let unitycatalog_client::Error::Api(api_err) = err {
        return napi::Error::from_reason(format!("UC:{}:{}", api_err.error_code(), api_err));
    }

    let mut message = err.to_string();

    // Append causes
    let mut cause = std::error::Error::source(err);
    let mut indent = 2;
    while let Some(e) = cause {
        let cause_message = format!("Caused by: {}", e);
        message.push_str(&indent_string(&cause_message, indent));

        cause = e.source();
        indent += 2;
    }

    napi::Error::from_reason(message)
}

fn indent_string(s: &str, amount: usize) -> String {
    let indent = " ".repeat(amount);
    s.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}
