//! Utility deserializers for flexible handling of UPV backend JSON variations.

use serde::{de, Deserialize, Deserializer};

/// Deserializes boolean values from multiple formats:
/// - bool: true / false
/// - number: 1 / 0
/// - string: "true", "false", "S", "s", "N", "n", "1", "0", "SI", "NO"
pub fn deserialize_flexible_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrString {
        Bool(bool),
        I64(i64),
        U64(u64),
        Str(String),
    }

    let opt: Option<BoolOrString> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(false),
        Some(BoolOrString::Bool(b)) => Ok(b),
        Some(BoolOrString::I64(i)) => Ok(i != 0),
        Some(BoolOrString::U64(u)) => Ok(u != 0),
        Some(BoolOrString::Str(v)) => {
            let trimmed = v.trim();
            if trimmed.eq_ignore_ascii_case("s")
                || trimmed.eq_ignore_ascii_case("si")
                || trimmed.eq_ignore_ascii_case("true")
                || trimmed == "1"
            {
                Ok(true)
            } else if trimmed.eq_ignore_ascii_case("n")
                || trimmed.eq_ignore_ascii_case("no")
                || trimmed.eq_ignore_ascii_case("false")
                || trimmed == "0"
                || trimmed.is_empty()
            {
                Ok(false)
            } else {
                Err(de::Error::custom(format!("invalid boolean string: {v}")))
            }
        }
    }
}

/// Deserializes optional f64 numbers that may arrive as float, int, or string (with dot or comma).
pub fn deserialize_flexible_float_opt<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FloatOrString {
        F64(f64),
        I64(i64),
        Str(String),
        Null,
    }

    let opt: Option<FloatOrString> = Option::deserialize(deserializer)?;
    match opt {
        None | Some(FloatOrString::Null) => Ok(None),
        Some(FloatOrString::F64(v)) => Ok(Some(v)),
        Some(FloatOrString::I64(v)) => Ok(Some(v as f64)),
        Some(FloatOrString::Str(s)) => {
            let part = s.split('/').next().unwrap_or(&s).trim();
            let cleaned = part.replace(',', ".");
            if cleaned.is_empty() || cleaned.eq_ignore_ascii_case("null") || cleaned == "-" {
                Ok(None)
            } else {
                cleaned
                    .parse::<f64>()
                    .map(Some)
                    .map_err(de::Error::custom)
            }
        }
    }
}

/// Deserializes attachments which can be a list of strings, URLs, or objects with `url`/`name`.
pub fn deserialize_attachments<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AttachmentItem {
        Str(String),
        Obj(serde_json::Value),
    }

    let items: Result<Vec<AttachmentItem>, _> = Deserialize::deserialize(deserializer);
    match items {
        Ok(list) => {
            let res = list
                .into_iter()
                .map(|item| match item {
                    AttachmentItem::Str(s) => s,
                    AttachmentItem::Obj(val) => {
                        if let Some(url) = val.get("url").and_then(|v| v.as_str()) {
                            url.to_string()
                        } else if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
                            name.to_string()
                        } else {
                            val.to_string()
                        }
                    }
                })
                .collect();
            Ok(res)
        }
        Err(_) => Ok(Vec::new()),
    }
}

/// Deserializes a field that could arrive as a string, number, or null into a String.
pub fn deserialize_flexible_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StrOrNum {
        Str(String),
        I64(i64),
        U64(u64),
        F64(f64),
    }

    let opt: Option<StrOrNum> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(String::new()),
        Some(StrOrNum::Str(s)) => Ok(s),
        Some(StrOrNum::I64(i)) => Ok(i.to_string()),
        Some(StrOrNum::U64(u)) => Ok(u.to_string()),
        Some(StrOrNum::F64(f)) => Ok(f.to_string()),
    }
}
