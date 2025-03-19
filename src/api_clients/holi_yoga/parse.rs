use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use scraper::{ElementRef, Html, Selector};

use crate::api_clients::holi_yoga::consts::{
    CLASS_INSTRUCTOR_CSS_SELECTOR, SCHEDULE_CLASS_INSTRUCTOR_CSS_SELECTOR,
    SCHEDULE_CLASS_NAME_CSS_SELECTOR, SCHEDULE_CLASS_TIME_CSS_SELECTOR,
};
use crate::api_clients::models::{Class, UtcDateTime};
use crate::api_clients::{errors::ClassParseError, holi_yoga::consts::CLASS_ID_CSS_SELECTOR};

use super::consts::{
    CLASSES_CSS_SELECTOR, CLASS_CSS_SELECTOR, CLASS_DATE_SELECTOR, CLASS_DURATION_SELECTOR,
    CLASS_NAME_CSS_SELECTOR, CLASS_START_TIME_SELECTOR, SCHEDULE_CSS_SELECTOR,
};

pub fn parse_user_classes(
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
        let id_element = get_element(&class_element, CLASS_ID_CSS_SELECTOR, "class id")?;
        let class_id: String = serde_json::from_str(&format!(
            "\"{}\"",
            id_element
                .attr("data-id")
                .ok_or(ClassParseError::MissingHtmlAttribute {
                    css_selector: CLASS_ID_CSS_SELECTOR.to_owned(),
                    attribute: "data-id".to_owned()
                })?
        ))
        .unwrap();

        let name_element = get_element(&class_element, CLASS_NAME_CSS_SELECTOR, "class name")?;
        let class_name = format!(
            "{}",
            name_element
                .inner_html()
                .split("<br>")
                .collect::<Vec<&str>>()
                .first()
                .unwrap()
                .trim()
        );

        let instructor_element = get_element(
            &class_element,
            CLASS_INSTRUCTOR_CSS_SELECTOR,
            "class instructor",
        )?;
        let instructor: String =
            serde_json::from_str(&format!("\"{}\"", instructor_element.inner_html().trim()))
                .unwrap();

        let date_element = get_element(&class_element, CLASS_DATE_SELECTOR, "class date")?;

        let start_time_element = get_element(
            &class_element,
            CLASS_START_TIME_SELECTOR,
            "class start time",
        )?;
        let duration = get_element(&class_element, CLASS_DURATION_SELECTOR, "class duration")?;
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
        let start = naivedate_to_utc(&start, time_zone);

        let end = start + Duration::seconds(60 * i64::from(duration_min));
        let class = Class {
            id: class_id,
            name: class_name,
            instructor,
            start,
            end,
        };
        log::debug!("Parsed class: {}", class);
        classes.push(class);
    }
    Ok(classes)
}

pub fn parse_schedule(
    html: &Html,
    date: NaiveDate,
    time_zone: chrono_tz::Tz,
) -> Result<Vec<Class>, ClassParseError<'static>> {
    let classes_selector = Selector::parse(SCHEDULE_CSS_SELECTOR).unwrap();
    let classes_element =
        html.select(&classes_selector)
            .next()
            .ok_or(ClassParseError::MissingCssSelector {
                field: "schedule".to_owned(),
                css_selector: SCHEDULE_CSS_SELECTOR,
            })?;

    let class_selector = Selector::parse("tr").unwrap();
    let mut classes = Vec::new();
    for class_element in classes_element.select(&class_selector) {
        let id_element = get_element(&class_element, CLASS_ID_CSS_SELECTOR, "class id")?;
        let class_id: String = serde_json::from_str(&format!(
            "\"{}\"",
            id_element
                .attr("data-id")
                .ok_or(ClassParseError::MissingHtmlAttribute {
                    css_selector: CLASS_ID_CSS_SELECTOR.to_owned(),
                    attribute: "data-id".to_owned()
                })?
        ))
        .unwrap();

        let name_element = get_element(
            &class_element,
            SCHEDULE_CLASS_NAME_CSS_SELECTOR,
            "class name",
        )?;
        let class_name: String = serde_json::from_str(&format!(
            "\"{}\"",
            name_element.inner_html().trim().replace("&amp;", "&")
        ))
        .unwrap();

        let instructor_element = get_element(
            &class_element,
            SCHEDULE_CLASS_INSTRUCTOR_CSS_SELECTOR,
            "class instructor",
        )?;
        let instructor: String =
            serde_json::from_str(&format!("\"{}\"", instructor_element.inner_html().trim()))
                .unwrap();

        let time = get_element(
            &class_element,
            SCHEDULE_CLASS_TIME_CSS_SELECTOR,
            "class time",
        )?
        .inner_html();
        let time = time.trim();

        let (start_time, end_time) = time.split_once('-').ok_or(ClassParseError::WrongFormat {
            field: "time".to_owned(),
            expected: "start-end".to_owned(),
        })?;

        let start =
            NaiveDateTime::parse_from_str(&format!("{date} {start_time}:00"), "%Y-%m-%d %H:%M:%S")?;
        let start = naivedate_to_utc(&start, time_zone);

        let end =
            NaiveDateTime::parse_from_str(&format!("{date} {end_time}:00"), "%Y-%m-%d %H:%M:%S")?;
        let end = naivedate_to_utc(&end, time_zone);

        let class = Class {
            id: class_id,
            name: class_name,
            instructor,
            start,
            end,
        };
        log::debug!("Parsed class: {}", class);
        classes.push(class);
    }
    Ok(classes)
}

fn get_element<'a>(
    element: &'a ElementRef,
    selector: &'static str,
    field: &str,
) -> Result<ElementRef<'a>, ClassParseError<'static>> {
    element
        .select(&Selector::parse(selector).unwrap())
        .next()
        .ok_or(ClassParseError::MissingCssSelector {
            field: field.to_owned(),
            css_selector: selector,
        })
}

fn naivedate_to_utc(naive: &NaiveDateTime, time_zone: chrono_tz::Tz) -> UtcDateTime {
    let time = time_zone.from_local_datetime(naive).unwrap();
    Utc.from_utc_datetime(&time.naive_utc())
}
