//! Network tool handlers.

use super::{get_string, optional_string, require_grant, DispatchError, DispatchResult};
use crate::capability::{CapabilityKind, Grant, HttpMethod, PrincipalId};
use chief_event_log_proto::EventLog;
use reqwest::{Method, Url};
use serde_json::{json, Map, Value};
use std::time::Duration;

pub(crate) async fn http(
    client: &reqwest::Client,
    args: &Value,
    principal: &PrincipalId,
    grants: &[Grant],
    handle_scope: Option<&CapabilityKind>,
    event_log: &EventLog,
) -> DispatchResult {
    let request = args.get("request").unwrap_or(args);
    let url_raw = get_string(request, "url")?;
    let url = Url::parse(url_raw)
        .map_err(|err| DispatchError::InvalidArguments(format!("invalid url {url_raw}: {err}")))?;
    let host = url
        .host_str()
        .ok_or_else(|| DispatchError::InvalidArguments("url has no host".to_string()))?;
    let method = parse_method(optional_string(request, "method").unwrap_or("GET"))?;
    let chief_method = chief_method(&method);

    require_grant(
        event_log,
        principal,
        grants,
        handle_scope,
        "net.http",
        |capability| {
            matches!(capability, CapabilityKind::NetHttp { hosts, methods, .. }
            if host_allowed(hosts, &url) && method_allowed(methods, &chief_method))
        },
    )?;

    if url.scheme() != "https" {
        let local_http_allowed = url.scheme() == "http" && is_localhost(host);
        if !local_http_allowed {
            return Err(DispatchError::CapabilityDenied(format!(
                "net.http denied non-https scheme `{}` for host {host}",
                url.scheme()
            )));
        }
    }

    let timeout = request
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_secs(10));
    let mut builder = client.request(method, url.clone()).timeout(timeout);

    if let Some(headers) = request.get("headers").and_then(Value::as_object) {
        for (name, value) in headers {
            let value = value.as_str().ok_or_else(|| {
                DispatchError::InvalidArguments(format!("header `{name}` value must be string"))
            })?;
            builder = builder.header(name, value);
        }
    }
    if let Some(body) = request.get("body") {
        builder = if let Some(text) = body.as_str() {
            builder.body(text.to_string())
        } else {
            builder.json(body)
        };
    }

    let response = builder
        .send()
        .await
        .map_err(|err| DispatchError::Http(err.to_string()))?;
    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), Value::String(value.to_string())))
        })
        .collect::<Map<String, Value>>();
    let body = response
        .text()
        .await
        .map_err(|err| DispatchError::Http(err.to_string()))?;

    Ok(json!({
        "status": "ok",
        "url": url.as_str(),
        "http_status": status,
        "headers": headers,
        "body": body,
    }))
}

fn parse_method(raw: &str) -> Result<Method, DispatchError> {
    raw.parse::<Method>()
        .map_err(|err| DispatchError::InvalidArguments(format!("invalid method {raw}: {err}")))
}

fn chief_method(method: &Method) -> HttpMethod {
    match *method {
        Method::GET => HttpMethod::Get,
        Method::POST => HttpMethod::Post,
        Method::PUT => HttpMethod::Put,
        Method::PATCH => HttpMethod::Patch,
        Method::DELETE => HttpMethod::Delete,
        _ => HttpMethod::Any,
    }
}

fn method_allowed(allowed: &[HttpMethod], requested: &HttpMethod) -> bool {
    allowed
        .iter()
        .any(|method| matches!(method, HttpMethod::Any) || method == requested)
}

fn host_allowed(patterns: &[String], url: &Url) -> bool {
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    let path = url.path();
    patterns
        .iter()
        .any(|pattern| pattern == "*" || host_pattern_matches(pattern, host, path))
}

fn host_pattern_matches(pattern: &str, host: &str, path: &str) -> bool {
    let without_scheme = pattern
        .strip_prefix("https://")
        .or_else(|| pattern.strip_prefix("http://"))
        .unwrap_or(pattern);
    let (host_pattern, path_prefix) = without_scheme
        .split_once('/')
        .map(|(host, path)| (host, Some(format!("/{path}"))))
        .unwrap_or((without_scheme, None));

    let host_matches = if let Some(suffix) = host_pattern.strip_prefix("*.") {
        host == suffix || host.ends_with(&format!(".{suffix}"))
    } else {
        host.eq_ignore_ascii_case(host_pattern)
    };
    let path_matches = path_prefix
        .as_deref()
        .map(|prefix| path.starts_with(prefix))
        .unwrap_or(true);
    host_matches && path_matches
}

fn is_localhost(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}
