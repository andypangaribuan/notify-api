/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use std::collections::HashMap;

use rmod::FCT;

#[derive(Debug, PartialEq, Clone)]
pub struct AppdataValue {
    pub int_value: Option<i32>,
    pub numeric_value: Option<FCT>,
    pub string_value: Option<String>,
    pub bool_value: Option<bool>,

    pub int_values: Option<Vec<i32>>,
    pub numeric_values: Option<Vec<FCT>>,
    pub strings_values: Option<Vec<String>>,
    pub bool_values: Option<Vec<bool>>,
    pub map_values: Option<HashMap<String, String>>,
}

impl AppdataValue {
    pub fn new(int_value: Option<i32>, numeric_value: Option<FCT>, string_value: Option<String>, bool_value: Option<bool>) -> Self {
        Self {
            int_value,
            numeric_value,
            string_value,
            bool_value,
            int_values: None,
            numeric_values: None,
            strings_values: None,
            bool_values: None,
            map_values: None,
        }
    }

    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            int_value: None,
            numeric_value: None,
            string_value: None,
            bool_value: None,
            int_values: None,
            numeric_values: None,
            strings_values: None,
            bool_values: None,
            map_values: None,
        }
    }
}
