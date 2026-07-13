/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use rmod::{fuse::FuseRContext, fuse::FuseResult, http::StatusCode, json, serde::Serialize};
use std::{any::Any, sync::Arc};

#[derive(Serialize)]
#[serde(crate = "rmod::serde")]
struct ApiResponse<T = json::Value> {
    code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<json::Value>,
}

#[track_caller]
pub fn response<T: Serialize>(
    ctx: &mut FuseRContext,
    status: StatusCode,
    sub_code: Option<&str>,
    message: Option<&str>,
    data: T,
    extra: Option<json::Value>,
) -> FuseResult {
    let (status, body) = dispatch_val(ctx, status, sub_code, message, data, extra);
    if status.is_success() { Ok((status, body)) } else { Err((status, body)) }
}

#[track_caller]
pub fn dispatch_val<T: Serialize>(
    ctx: &mut FuseRContext,
    status: StatusCode,
    sub_code: Option<&str>,
    message: Option<&str>,
    data: T,
    extra: Option<json::Value>,
) -> (StatusCode, Arc<dyn Any + Send + Sync>) {
    let val = json::to_value(data).unwrap();
    let sub_code = sub_code
        .map(|s| s.to_string())
        .or_else(|| if status == StatusCode::INTERNAL_SERVER_ERROR { Some("internal-server-error".to_string()) } else { None });

    let body = ApiResponse {
        code: status.as_u16(),
        sub_code,
        message: message.map(|s| s.to_string()),
        data: if val.is_null() { None } else { Some(val) },
        extra: match extra {
            Some(json::Value::Object(ref m)) if !m.is_empty() => extra,
            _ => None,
        },
    };

    let body_val = json::to_value(body).unwrap_or_else(|_| {
        json::json!({
            "code": status.as_u16(),
            "sub_code": "internal-server-error",
            "message": "internal serialization error"
        })
    });

    ctx.err_val(status, body_val)
}

#[macro_export]
macro_rules! json_response {
    ($ctx:expr, $status:expr) => {
        $crate::ext::response::response($ctx, $status, None, None, (), None)
    };
    ($ctx:expr, $status:expr, sub=$sub:expr) => {
        $crate::ext::response::response($ctx, $status, Some($sub.as_ref()), None, (), None)
    };
    ($ctx:expr, $status:expr, msg=$msg:expr) => {
        $crate::ext::response::response($ctx, $status, None, Some($msg.as_ref()), (), None)
    };
    ($ctx:expr, $status:expr, sub=$sub:expr, msg=$msg:expr) => {
        $crate::ext::response::response($ctx, $status, Some($sub.as_ref()), Some($msg.as_ref()), (), None)
    };
    ($ctx:expr, $status:expr, data=$data:tt) => {
        $crate::ext::response::response($ctx, $status, None, None, ::rmod::json::json!($data), None)
    };
    ($ctx:expr, $status:expr, msg=$msg:expr, data=$data:tt) => {
        $crate::ext::response::response($ctx, $status, None, Some($msg.as_ref()), ::rmod::json::json!($data), None)
    };
    ($ctx:expr, $status:expr, sub=$sub:expr, msg=$msg:expr, data=$data:tt) => {
        $crate::ext::response::response($ctx, $status, Some($sub.as_ref()), Some($msg.as_ref()), ::rmod::json::json!($data), None)
    };

    // Dynamic field variants
    ($ctx:expr, $status:expr, sub=$sub:expr, msg=$msg:expr, data=$data:tt, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::response($ctx, $status, Some($sub.as_ref()), Some($msg.as_ref()), ::rmod::json::json!($data), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, sub=$sub:expr, msg=$msg:expr, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::response($ctx, $status, Some($sub.as_ref()), Some($msg.as_ref()), (), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, sub=$sub:expr, data=$data:tt, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::response($ctx, $status, Some($sub.as_ref()), None, ::rmod::json::json!($data), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, sub=$sub:expr, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::response($ctx, $status, Some($sub.as_ref()), None, (), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, msg=$msg:expr, data=$data:tt, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::response($ctx, $status, None, Some($msg.as_ref()), ::rmod::json::json!($data), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, msg=$msg:expr, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::response($ctx, $status, None, Some($msg.as_ref()), (), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, data=$data:tt, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::response($ctx, $status, None, None, ::rmod::json::json!($data), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, $k:ident = $v:tt $(, $next_k:ident = $next_v:tt)* $(,)?) => {
        $crate::ext::response::response($ctx, $status, None, None, (), Some(::rmod::json::json!({ stringify!($k): $v $(, stringify!($next_k): $next_v)* })))
    };
}

#[macro_export]
macro_rules! dispatch_response {
    ($ctx:expr, $status:expr) => {
        $crate::ext::response::dispatch_val($ctx, $status, None, None, (), None)
    };
    ($ctx:expr, $status:expr, sub=$sub:expr) => {
        $crate::ext::response::dispatch_val($ctx, $status, Some($sub.as_ref()), None, (), None)
    };
    ($ctx:expr, $status:expr, msg=$msg:expr) => {
        $crate::ext::response::dispatch_val($ctx, $status, None, Some($msg.as_ref()), (), None)
    };
    ($ctx:expr, $status:expr, sub=$sub:expr, msg=$msg:expr) => {
        $crate::ext::response::dispatch_val($ctx, $status, Some($sub.as_ref()), Some($msg.as_ref()), (), None)
    };
    ($ctx:expr, $status:expr, data=$data:tt) => {
        $crate::ext::response::dispatch_val($ctx, $status, None, None, ::rmod::json::json!($data), None)
    };
    ($ctx:expr, $status:expr, msg=$msg:expr, data=$data:tt) => {
        $crate::ext::response::dispatch_val($ctx, $status, None, Some($msg.as_ref()), ::rmod::json::json!($data), None)
    };
    ($ctx:expr, $status:expr, sub=$sub:expr, msg=$msg:expr, data=$data:tt) => {
        $crate::ext::response::dispatch_val($ctx, $status, Some($sub.as_ref()), Some($msg.as_ref()), ::rmod::json::json!($data), None)
    };

    // Dynamic field variants
    ($ctx:expr, $status:expr, sub=$sub:expr, msg=$msg:expr, data=$data:tt, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::dispatch_val($ctx, $status, Some($sub.as_ref()), Some($msg.as_ref()), ::rmod::json::json!($data), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, sub=$sub:expr, msg=$msg:expr, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::dispatch_val($ctx, $status, Some($sub.as_ref()), Some($msg.as_ref()), (), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, sub=$sub:expr, data=$data:tt, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::dispatch_val($ctx, $status, Some($sub.as_ref()), None, ::rmod::json::json!($data), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, sub=$sub:expr, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::dispatch_val($ctx, $status, Some($sub.as_ref()), None, (), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, msg=$msg:expr, data=$data:tt, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::dispatch_val($ctx, $status, None, Some($msg.as_ref()), ::rmod::json::json!($data), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, msg=$msg:expr, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::dispatch_val($ctx, $status, None, Some($msg.as_ref()), (), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, data=$data:tt, $($k:ident = $v:tt),+ $(,)?) => {
        $crate::ext::response::dispatch_val($ctx, $status, None, None, ::rmod::json::json!($data), Some(::rmod::json::json!({ $(stringify!($k): $v),* })))
    };
    ($ctx:expr, $status:expr, $k:ident = $v:tt $(, $next_k:ident = $next_v:tt)* $(,)?) => {
        $crate::ext::response::dispatch_val($ctx, $status, None, None, (), Some(::rmod::json::json!({ stringify!($k): $v $(, stringify!($next_k): $next_v)* })))
    };
}

pub trait FuseRContextExt {
    #[allow(dead_code)]
    fn unwrap_fetch<T, E: std::fmt::Debug, F>(
        &mut self,
        res: Result<Option<T>, E>,
        f: F,
    ) -> Result<T, (StatusCode, Arc<dyn Any + Send + Sync>)>
    where
        F: FnOnce(&mut FuseRContext) -> FuseResult;

    #[allow(dead_code)]
    fn unwrap_fetch_all<T, E: std::fmt::Debug, F>(
        &mut self,
        res: Result<Vec<T>, E>,
        f: F,
    ) -> Result<Vec<T>, (StatusCode, Arc<dyn Any + Send + Sync>)>
    where
        F: FnOnce(&mut FuseRContext) -> FuseResult;

    #[allow(dead_code)]
    fn unwrap_fetch_opt<T, E: std::fmt::Debug, F>(
        &mut self,
        res: Result<T, E>,
        f: F,
    ) -> Result<T, (StatusCode, Arc<dyn Any + Send + Sync>)>
    where
        F: FnOnce(&mut FuseRContext) -> FuseResult;
}

impl FuseRContextExt for FuseRContext {
    #[allow(dead_code)]
    fn unwrap_fetch<T, E: std::fmt::Debug, F>(
        &mut self,
        res: Result<Option<T>, E>,
        f: F,
    ) -> Result<T, (StatusCode, Arc<dyn Any + Send + Sync>)>
    where
        F: FnOnce(&mut FuseRContext) -> FuseResult,
    {
        match res {
            Ok(Some(v)) => Ok(v),
            Ok(None) => match f(self) {
                Ok(v) => Err(v),
                Err(v) => Err(v),
            },
            Err(e) => {
                let msg = format!("{:#?}", e);
                Err(dispatch_val(self, StatusCode::INTERNAL_SERVER_ERROR, Some("db_error"), Some(&msg), (), None))
            }
        }
    }

    #[allow(dead_code)]
    fn unwrap_fetch_all<T, E: std::fmt::Debug, F>(
        &mut self,
        res: Result<Vec<T>, E>,
        f: F,
    ) -> Result<Vec<T>, (StatusCode, Arc<dyn Any + Send + Sync>)>
    where
        F: FnOnce(&mut FuseRContext) -> FuseResult,
    {
        match res {
            Ok(v) => Ok(v),
            Err(e) => {
                let msg = format!("{:#?}", e);
                let _ = dispatch_val(self, StatusCode::INTERNAL_SERVER_ERROR, Some("db_error"), Some(&msg), (), None);
                match f(self) {
                    Ok(v) => Err(v),
                    Err(v) => Err(v),
                }
            }
        }
    }

    #[allow(dead_code)]
    fn unwrap_fetch_opt<T, E: std::fmt::Debug, F>(&mut self, res: Result<T, E>, f: F) -> Result<T, (StatusCode, Arc<dyn Any + Send + Sync>)>
    where
        F: FnOnce(&mut FuseRContext) -> FuseResult,
    {
        match res {
            Ok(v) => Ok(v),
            Err(e) => {
                let msg = format!("{:#?}", e);
                let _ = dispatch_val(self, StatusCode::INTERNAL_SERVER_ERROR, Some("db_error"), Some(&msg), (), None);
                match f(self) {
                    Ok(v) => Err(v),
                    Err(v) => Err(v),
                }
            }
        }
    }
}
