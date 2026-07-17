use std::sync::LazyLock;

use super::model::WarpThemeData;
use super::parser::parse_theme;

pub struct EmbeddedWarpThemeSource {
    pub id: &'static str,
    pub category: &'static str,
    pub stem: &'static str,
    pub yaml: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/warp_theme_catalog.rs"));

#[derive(Debug, Clone)]
pub struct CatalogTheme {
    pub id: &'static str,
    pub category: &'static str,
    pub stem: &'static str,
    pub display_name: String,
    pub data: WarpThemeData,
    pub content_hash: [u8; 32],
}

static CATALOG: LazyLock<Vec<CatalogTheme>> = LazyLock::new(|| {
    EMBEDDED_WARP_THEMES
        .iter()
        .filter_map(|source| match parse_theme(source.yaml) {
            Ok(data) => {
                let display_name = data.name.clone().unwrap_or_else(|| humanize(source.stem));
                Some(CatalogTheme {
                    id: source.id,
                    category: source.category,
                    stem: source.stem,
                    display_name,
                    data,
                    content_hash: *blake3::hash(source.yaml.as_bytes()).as_bytes(),
                })
            }
            Err(error) => {
                tracing::warn!(theme_id = source.id, %error, "invalid embedded Warp theme");
                None
            }
        })
        .collect()
});

pub fn all() -> &'static [CatalogTheme] {
    CATALOG.as_slice()
}

pub fn find(id: &str) -> Option<&'static CatalogTheme> {
    all().iter().find(|theme| theme.id == id)
}

pub fn find_by_warp_name(name: &str) -> Option<&'static CatalogTheme> {
    let wanted = normalized_name(name);
    all()
        .iter()
        .filter(|theme| {
            normalized_name(theme.stem) == wanted || normalized_name(&theme.display_name) == wanted
        })
        .min_by_key(|theme| category_priority(theme.category))
}

fn category_priority(category: &str) -> u8 {
    match category {
        "warp_bundled" => 0,
        "standard" => 1,
        "special_edition" => 2,
        "stradicat" => 3,
        "base16" => 4,
        _ => 5,
    }
}

pub fn normalized_name(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn humanize(stem: &str) -> String {
    stem.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_catalog_parses() {
        assert_eq!(EMBEDDED_WARP_THEMES.len(), 340);
        assert_eq!(all().len(), 340);
    }

    #[test]
    fn ids_are_unique() {
        let mut ids = all().iter().map(|theme| theme.id).collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 340);
    }

    #[test]
    fn bundled_theme_wins_duplicate_name() {
        let dracula = find_by_warp_name("dracula").expect("Dracula");
        assert_eq!(dracula.id, "warp_bundled/dracula");
    }

    #[test]
    fn gradient_and_image_metadata_survive() {
        let fancy = find("warp_bundled/fancy_dracula").expect("Fancy Dracula");
        assert!(fancy.data.is_gradient());
        let leafy = find("warp_bundled/leafy").expect("Leafy");
        assert!(leafy.data.has_background_image);
    }
}
