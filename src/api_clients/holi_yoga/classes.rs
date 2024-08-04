use chrono::{Datelike, Duration, NaiveDateTime, TimeZone, Utc};
use scraper::{Html, Selector};

use crate::api_clients::errors::ClassParseError;
use crate::api_clients::models::Class;

use super::consts::{
    CLASSES_CSS_SELECTOR, CLASS_CSS_SELECTOR, CLASS_DATE_SELECTOR, CLASS_DURATION_SELECTOR,
    CLASS_NAME_CSS_SELECTOR, CLASS_START_TIME_SELECTOR,
};

pub fn parse_classes(
    html: &Html,
    time_zone: chrono_tz::Tz,
) -> Result<Vec<Class>, ClassParseError<'static>> {
    let classes_selector = Selector::parse(CLASSES_CSS_SELECTOR).unwrap();
    let classes_element =
        html.select(&classes_selector)
            .next()
            .ok_or(ClassParseError::MissingCssSelector {
                field: "classes".to_owned(),
                css_selector: CLASSES_CSS_SELECTOR,
            })?;

    let class_selector = Selector::parse(CLASS_CSS_SELECTOR).unwrap();
    let mut classes = Vec::new();
    for class_element in classes_element.select(&class_selector) {
        let name_element = class_element
            .select(&Selector::parse(CLASS_NAME_CSS_SELECTOR).unwrap())
            .next()
            .ok_or(ClassParseError::MissingCssSelector {
                field: "class name".to_owned(),
                css_selector: CLASS_NAME_CSS_SELECTOR,
            })?;
        let class_name: String =
            serde_json::from_str(&format!("\"{}\"", name_element.inner_html().trim())).unwrap();

        let date_element = class_element
            .select(&Selector::parse(CLASS_DATE_SELECTOR).unwrap())
            .next()
            .ok_or(ClassParseError::MissingCssSelector {
                field: "class date".to_owned(),
                css_selector: CLASS_DATE_SELECTOR,
            })?;

        let start_time_element = class_element
            .select(&Selector::parse(CLASS_START_TIME_SELECTOR).unwrap())
            .next()
            .ok_or(ClassParseError::MissingCssSelector {
                field: "class start time".to_owned(),
                css_selector: CLASS_START_TIME_SELECTOR,
            })?;
        let duration = class_element
            .select(&Selector::parse(CLASS_DURATION_SELECTOR).unwrap())
            .next()
            .ok_or(ClassParseError::MissingCssSelector {
                field: "class duration".to_owned(),
                css_selector: CLASS_DURATION_SELECTOR,
            })?;
        let duration_min: u8 = duration
            .inner_html()
            .trim()
            .trim_end_matches("min")
            .trim()
            .parse()
            .map_err(|_| ClassParseError::WrongFormat {
                field: "class duration".to_owned(),
                expected: "<duration> min".to_owned(),
            })?;

        let start = NaiveDateTime::parse_from_str(
            &format!(
                "{} {} {}",
                chrono::Local::now().year(),
                date_element.inner_html().trim(),
                start_time_element.inner_html().trim(),
            ),
            "%Y %d %B, %A %H:%M",
        )
        .map_err(|err| ClassParseError::Other { error: err.into() })?;
        let start = time_zone.from_local_datetime(&start).unwrap();
        let start = Utc.from_utc_datetime(&start.naive_utc());

        let end = start + Duration::seconds(60 * i64::from(duration_min));
        let class = Class {
            name: class_name,
            start,
            end,
        };
        log::debug!("Parsed class: {}", class);
        classes.push(class);
    }
    Ok(classes)
}
