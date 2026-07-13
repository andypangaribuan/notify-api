/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{ext::var, model};
use rmod::FCT;

pub trait FromAppdataValue: Sized {
    fn from_val(val: &model::AppdataValue) -> Option<Self>;
}

impl FromAppdataValue for i32 {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.int_value
    }
}

impl FromAppdataValue for u32 {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.int_value.and_then(|v| u32::try_from(v).ok())
    }
}

impl FromAppdataValue for FCT {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.numeric_value
    }
}

impl FromAppdataValue for String {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.string_value.clone()
    }
}

impl FromAppdataValue for bool {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.bool_value
    }
}

impl FromAppdataValue for Vec<i32> {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.int_values.clone()
    }
}

impl FromAppdataValue for i64 {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.int_value.map(i64::from)
    }
}

impl FromAppdataValue for Vec<u32> {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.int_values.as_ref().and_then(|v| v.iter().map(|&x| u32::try_from(x)).collect::<Result<Vec<u32>, _>>().ok())
    }
}

impl FromAppdataValue for Vec<FCT> {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.numeric_values.clone()
    }
}

impl FromAppdataValue for Vec<String> {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.strings_values.clone()
    }
}

impl FromAppdataValue for Vec<bool> {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.bool_values.clone()
    }
}

impl FromAppdataValue for std::collections::HashMap<String, String> {
    fn from_val(val: &model::AppdataValue) -> Option<Self> {
        val.map_values.clone()
    }
}

pub fn get_appdata<T: FromAppdataValue>(env_app: &str, key: &str) -> Option<T> {
    let store = var::appdatas().read().unwrap();
    store.get(env_app).and_then(|appdatas| appdatas.get(key)).and_then(|val| T::from_val(val))
}

pub trait FromAppdataVecValue: Sized {
    fn from_vec_val(val: &model::AppdataValue, separator: &str) -> Option<Vec<Self>>;
}

impl FromAppdataVecValue for String {
    fn from_vec_val(val: &model::AppdataValue, separator: &str) -> Option<Vec<Self>> {
        if let Some(ref list) = val.strings_values {
            Some(list.clone())
        } else if let Some(ref s) = val.string_value {
            Some(s.split(separator).map(|x| x.trim().to_string()).collect())
        } else {
            None
        }
    }
}

impl FromAppdataVecValue for FCT {
    fn from_vec_val(val: &model::AppdataValue, separator: &str) -> Option<Vec<Self>> {
        if let Some(ref list) = val.numeric_values {
            Some(list.clone())
        } else if let Some(ref s) = val.string_value {
            s.split(separator)
                .map(|x| x.trim().parse::<FCT>().ok())
                .collect()
        } else {
            None
        }
    }
}

impl FromAppdataVecValue for i32 {
    fn from_vec_val(val: &model::AppdataValue, separator: &str) -> Option<Vec<Self>> {
        if let Some(ref list) = val.int_values {
            Some(list.clone())
        } else if let Some(ref s) = val.string_value {
            s.split(separator)
                .map(|x| x.trim().parse::<i32>().ok())
                .collect()
        } else {
            None
        }
    }
}

impl FromAppdataVecValue for u32 {
    fn from_vec_val(val: &model::AppdataValue, separator: &str) -> Option<Vec<Self>> {
        if let Some(ref list) = val.int_values {
            list.iter().map(|&x| u32::try_from(x).ok()).collect()
        } else if let Some(ref s) = val.string_value {
            s.split(separator)
                .map(|x| x.trim().parse::<u32>().ok())
                .collect()
        } else {
            None
        }
    }
}

impl FromAppdataVecValue for bool {
    fn from_vec_val(val: &model::AppdataValue, separator: &str) -> Option<Vec<Self>> {
        if let Some(ref list) = val.bool_values {
            Some(list.clone())
        } else if let Some(ref s) = val.string_value {
            s.split(separator)
                .map(|x| x.trim().parse::<bool>().ok())
                .collect()
        } else {
            None
        }
    }
}

pub fn get_vec_appdata<T: FromAppdataVecValue>(env_app: &str, key: &str, separator: &str) -> Option<Vec<T>> {
    let store = var::appdatas().read().unwrap();
    let val = store.get(env_app).and_then(|appdatas| appdatas.get(key))?;
    T::from_vec_val(val, separator)
}
