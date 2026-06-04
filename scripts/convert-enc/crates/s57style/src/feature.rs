use serde_json::Value;

pub type PropMap = std::collections::HashMap<String, Value>;

/// Extension trait for reading S-57 feature properties from a JSON property map.
/// Values are permissive: strings containing numbers are parsed as numbers, etc.
pub trait PropMapExt {
    fn get_str(&self, key: &str) -> Option<&str>;
    fn get_int(&self, key: &str) -> Option<i64>;
    fn get_float(&self, key: &str) -> Option<f64>;
    /// Reads the key as a list of integers. Handles JSON arrays and single values.
    fn get_int_list(&self, key: &str) -> Vec<i64>;
}

impl PropMapExt for PropMap {
    fn get_str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn get_int(&self, key: &str) -> Option<i64> {
        match self.get(key)? {
            Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
            Value::String(s) => s.trim().parse::<i64>().ok().or_else(|| {
                s.trim().parse::<f64>().ok().map(|f| f as i64)
            }),
            _ => None,
        }
    }

    fn get_float(&self, key: &str) -> Option<f64> {
        match self.get(key)? {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.trim().parse::<f64>().ok(),
            _ => None,
        }
    }

    fn get_int_list(&self, key: &str) -> Vec<i64> {
        match self.get(key) {
            Some(Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| match v {
                    Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
                    Value::String(s) => s.trim().parse::<i64>().ok().or_else(|| {
                        s.trim().parse::<f64>().ok().map(|f| f as i64)
                    }),
                    _ => None,
                })
                .collect(),
            Some(Value::Number(n)) => {
                if let Some(i) = n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)) {
                    vec![i]
                } else {
                    vec![]
                }
            }
            Some(Value::String(s)) => {
                // Could be comma-separated or a single value
                if s.contains(',') {
                    s.split(',')
                        .filter_map(|p| p.trim().parse::<i64>().ok().or_else(|| {
                            p.trim().parse::<f64>().ok().map(|f| f as i64)
                        }))
                        .collect()
                } else {
                    s.trim().parse::<i64>().ok()
                        .or_else(|| s.trim().parse::<f64>().ok().map(|f| f as i64))
                        .map(|i| vec![i])
                        .unwrap_or_default()
                }
            }
            _ => vec![],
        }
    }
}
