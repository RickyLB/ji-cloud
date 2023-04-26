use super::ParseUrlExt;
use anyhow::Context;
use futures_signals::signal::Mutable;
use regex::Regex;
use shared::domain::module::body::_groups::design::{PuzzelEmbed as RawPuzzelEmbed, PuzzelId};
use utils::unwrap::UnwrapJiExt;

#[derive(Clone, Debug, Default)]
pub struct PartialPuzzelEmbed {
    pub url: Mutable<Option<PuzzelId>>,
}
impl PartialPuzzelEmbed {
    pub fn full(&self) -> anyhow::Result<PuzzelEmbed> {
        Ok(PuzzelEmbed {
            url: Mutable::new(self.url.get_cloned().context("")?),
        })
    }
}

#[derive(Clone, Debug)]
pub struct PuzzelEmbed {
    pub url: Mutable<PuzzelId>,
}
impl PuzzelEmbed {
    pub fn partial(&self) -> PartialPuzzelEmbed {
        PartialPuzzelEmbed {
            url: Mutable::new(Some(self.url.get_cloned())),
        }
    }
    pub fn update_from_partial(&self, partial: &PartialPuzzelEmbed) -> anyhow::Result<()> {
        self.url.set_neq(partial.url.get_cloned().context("")?);
        Ok(())
    }
}
impl From<RawPuzzelEmbed> for PuzzelEmbed {
    fn from(value: RawPuzzelEmbed) -> Self {
        Self {
            url: Mutable::new(value.url),
        }
    }
}
impl From<&PuzzelEmbed> for RawPuzzelEmbed {
    fn from(value: &PuzzelEmbed) -> Self {
        Self {
            url: value.url.get_cloned(),
        }
    }
}

impl ParseUrlExt for PuzzelId {
    fn try_parse(text: String) -> anyhow::Result<Self> {
        match get_id_from_url(&text) {
            Ok(_) => Ok(Self(text)),
            Err(_) => Err(anyhow::anyhow!("")),
        }
    }

    fn get_id(&self) -> &str {
        get_id_from_url(&self.0).expect_ji("Not a valid Puzzel url")
    }
}

const SHARE_URL_BASE: &str = "https://www.puzzel.com/card/";
const EMBED_IFRAME_BASE: &str = "<iframe ";
const ID_LENGTH: usize = 19;

fn get_id_from_url(url: &str) -> Result<&str, ()> {
    let id;
    log::info!("{url}");

    if is_id(url) {
        log::info!("here");
        return Ok(url);
    // } else if url.starts_with(SHARE_URL_BASE) {
    } else if url.starts_with(SHARE_URL_BASE) && url.len() >= SHARE_URL_BASE.len() + ID_LENGTH {
        log::info!("here");
        id = extract_id_share(url);
    } else if url.starts_with(EMBED_IFRAME_BASE) {
        log::info!("here");
        id = extract_id_iframe(url);
    } else {
        log::info!("here");
        return Err(());
    };
    log::info!("{id}");

    if is_id(id) {
        Ok(id)
    } else {
        Err(())
    }
}

fn extract_id_share(url: &str) -> &str {
    let base_length: usize = SHARE_URL_BASE.len();
    log::info!("{url}");
    &url[base_length..(base_length + ID_LENGTH)]
}

fn extract_id_iframe(code: &str) -> &str {
    let id_index = code.find(SHARE_URL_BASE).unwrap_ji() + SHARE_URL_BASE.len();
    log::info!("{code}");
    &code[id_index..(id_index + ID_LENGTH)]
}

fn is_id(id: &str) -> bool {
    let regex = Regex::new(&format!("^[a-z|0-9]{{{ID_LENGTH}}}$")).unwrap_ji();
    regex.is_match(id)
}

#[cfg(test)]
mod tests {
    use shared::domain::module::body::_groups::design::PuzzelId;

    use crate::stickers::embed::types::ParseUrlExt;

    #[test]
    fn can_get_id_from_url() {
        let valid_url_vec = vec![
            r#"<iframe src='https://puzzel.org/en/crossword/embed?p=-Mcv2dXz8i0Xj9YWeLfN' width="1200" height="780"  frameborder='0'></iframe>"#,
            "https://puzzel.org/en/crossword/embed?p=-Mcv2dXz8i0Xj9YWeLfN",
            "-Mcv2dXz8i0Xj9YWeLfN",
        ];

        for url in valid_url_vec {
            let id = PuzzelId::try_parse(url.to_string());

            assert!(id.is_ok());
        }
    }
}
