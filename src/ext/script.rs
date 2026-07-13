/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[macro_export]
macro_rules! unwrap_or_return {
    ($res:expr, $log_msg:expr) => {
        match $res {
            Ok(v) => v,
            Err(e) => {
                rmod::log!("{}: {:#?}", $log_msg, e);
                return;
            }
        }
    };
}
