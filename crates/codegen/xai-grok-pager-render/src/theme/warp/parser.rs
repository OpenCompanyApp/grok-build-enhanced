use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use serde_yaml::Value;

use super::model::{AnsiPalette, Fill, Rgb, WarpThemeData};

pub const MAX_THEME_BYTES: usize = 256 * 1024;
const MAX_VALUE_DEPTH: usize = 32;

#[derive(Debug, Deserialize)]
struct RawTheme {
    background: Value,
    foreground: String,
    accent: Value,
    #[serde(default)]
    cursor: Option<Value>,
    terminal_colors: RawTerminalColors,
    #[serde(default)]
    details: Option<Value>,
    #[serde(default)]
    background_image: Option<Value>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTerminalColors {
    normal: RawAnsiColors,
    bright: RawAnsiColors,
}

#[derive(Debug, Deserialize)]
struct RawAnsiColors {
    black: String,
    red: String,
    green: String,
    yellow: String,
    blue: String,
    magenta: String,
    cyan: String,
    white: String,
}

impl RawAnsiColors {
    fn parse(self) -> Result<[Rgb; 8]> {
        Ok([
            parse_hex(&self.black)?,
            parse_hex(&self.red)?,
            parse_hex(&self.green)?,
            parse_hex(&self.yellow)?,
            parse_hex(&self.blue)?,
            parse_hex(&self.magenta)?,
            parse_hex(&self.cyan)?,
            parse_hex(&self.white)?,
        ])
    }
}

pub fn parse_theme(yaml: &str) -> Result<WarpThemeData> {
    if yaml.len() > MAX_THEME_BYTES {
        bail!("Warp theme exceeds {MAX_THEME_BYTES} byte limit");
    }
    let value: Value = serde_yaml::from_str(yaml).context("invalid Warp theme YAML")?;
    if value_depth(&value) > MAX_VALUE_DEPTH {
        bail!("Warp theme nesting exceeds {MAX_VALUE_DEPTH} levels");
    }
    let raw: RawTheme = serde_yaml::from_value(value).context("invalid Warp theme fields")?;

    Ok(WarpThemeData {
        name: raw.name.filter(|name| !name.trim().is_empty()),
        background: parse_fill(&raw.background).context("invalid background")?,
        foreground: parse_hex(&raw.foreground).context("invalid foreground")?,
        accent: parse_fill(&raw.accent).context("invalid accent")?,
        cursor: raw
            .cursor
            .as_ref()
            .map(parse_fill)
            .transpose()
            .context("invalid cursor")?,
        terminal: AnsiPalette {
            normal: raw.terminal_colors.normal.parse()?,
            bright: raw.terminal_colors.bright.parse()?,
        },
        details: raw.details.as_ref().map(details_label),
        has_background_image: raw.background_image.is_some(),
    })
}

pub fn parse_hex(raw: &str) -> Result<Rgb> {
    let value = raw.trim().strip_prefix('#').unwrap_or(raw.trim());
    let expanded;
    let six = match value.len() {
        3 => {
            let mut s = String::with_capacity(6);
            for ch in value.chars() {
                if !ch.is_ascii_hexdigit() {
                    bail!("invalid hexadecimal color {raw:?}");
                }
                s.push(ch);
                s.push(ch);
            }
            expanded = s;
            expanded.as_str()
        }
        6 if value.chars().all(|ch| ch.is_ascii_hexdigit()) => value,
        _ => bail!("expected #RGB or #RRGGBB, got {raw:?}"),
    };
    Ok(Rgb::new(
        u8::from_str_radix(&six[0..2], 16)?,
        u8::from_str_radix(&six[2..4], 16)?,
        u8::from_str_radix(&six[4..6], 16)?,
    ))
}

fn parse_fill(value: &Value) -> Result<Fill> {
    match value {
        Value::String(color) => Ok(Fill::Solid(parse_hex(color)?)),
        Value::Mapping(map) => {
            let get = |key: &str| {
                map.get(Value::String(key.to_owned()))
                    .and_then(Value::as_str)
                    .map(parse_hex)
                    .transpose()
            };
            if let (Some(top), Some(bottom)) = (get("top")?, get("bottom")?) {
                return Ok(Fill::Vertical { top, bottom });
            }
            if let (Some(left), Some(right)) = (get("left")?, get("right")?) {
                return Ok(Fill::Horizontal { left, right });
            }
            Err(anyhow!(
                "gradient must contain top+bottom or left+right colors"
            ))
        }
        _ => Err(anyhow!("fill must be a hex color or gradient mapping")),
    }
}

fn details_label(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Mapping(map) if map.contains_key(Value::String("custom".to_owned())) => {
            "custom".to_owned()
        }
        _ => "custom".to_owned(),
    }
}

fn value_depth(value: &Value) -> usize {
    match value {
        Value::Sequence(values) => 1 + values.iter().map(value_depth).max().unwrap_or_default(),
        Value::Mapping(map) => {
            1 + map
                .iter()
                .flat_map(|(key, value)| [value_depth(key), value_depth(value)])
                .max()
                .unwrap_or_default()
        }
        Value::Tagged(tagged) => 1 + value_depth(&tagged.value),
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme_yaml(background: &str, accent: &str, extra: &str) -> String {
        format!(
            r#"
name: Fixture
background: {background}
foreground: '#ffffff'
accent: {accent}
cursor: '#0f0'
terminal_colors:
  normal:
    black: '#000000'
    red: '#ff0000'
    green: '#00ff00'
    yellow: '#ffff00'
    blue: '#0000ff'
    magenta: '#ff00ff'
    cyan: '#00ffff'
    white: '#ffffff'
  bright:
    black: '#111111'
    red: '#ee0000'
    green: '#00ee00'
    yellow: '#eeee00'
    blue: '#0000ee'
    magenta: '#ee00ee'
    cyan: '#00eeee'
    white: '#eeeeee'
{extra}
"#
        )
    }

    #[test]
    fn accepts_short_and_long_hex() {
        assert_eq!(parse_hex("#abc").unwrap(), Rgb::new(0xaa, 0xbb, 0xcc));
        assert_eq!(parse_hex("7AA2F7").unwrap(), Rgb::new(0x7a, 0xa2, 0xf7));
    }

    #[test]
    fn rejects_bad_hex() {
        assert!(parse_hex("#12").is_err());
        assert!(parse_hex("#ggg").is_err());
        assert!(parse_hex("#12345678").is_err());
    }

    #[test]
    fn parses_gradients_and_metadata_without_loading_images() {
        let yaml = theme_yaml(
            "{ top: '#000000', bottom: '#202020' }",
            "{ left: '#ff0000', right: '#0000ff' }",
            "details: { custom: true }\nbackground_image: { path: '/does/not/exist.jpg' }",
        );
        let parsed = parse_theme(&yaml).unwrap();
        assert!(matches!(parsed.background, Fill::Vertical { .. }));
        assert!(matches!(parsed.accent, Fill::Horizontal { .. }));
        assert_eq!(parsed.cursor, Some(Fill::Solid(Rgb::new(0, 255, 0))));
        assert_eq!(parsed.details.as_deref(), Some("custom"));
        assert!(parsed.has_background_image);
    }

    #[test]
    fn rejects_missing_required_palette_fields() {
        let yaml = theme_yaml("'#000000'", "'#ffffff'", "");
        let yaml = yaml.replace("    cyan: '#00eeee'\n", "");
        assert!(parse_theme(&yaml).is_err());
    }

    #[test]
    fn rejects_excessive_yaml_depth() {
        let mut nested = "value".to_owned();
        for _ in 0..=MAX_VALUE_DEPTH {
            nested = format!("[{nested}]");
        }
        let yaml = theme_yaml("'#000000'", "'#ffffff'", &format!("future: {nested}"));
        assert!(
            parse_theme(&yaml)
                .unwrap_err()
                .to_string()
                .contains("nesting")
        );
    }
}
