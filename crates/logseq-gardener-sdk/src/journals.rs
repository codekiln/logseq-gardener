//! Journal dates, titles and date references, as Logseq derives them.
//!
//! ```
//! use logseq_gardener_sdk::journals::{DateFormat, JournalDate, JournalFormats, parse_journal_reference};
//!
//! let title = DateFormat::parse("yyyy-MM-dd EEE").unwrap();
//! let date = JournalDate::new(2026, 10, 23).unwrap();
//! assert_eq!(title.format(date), "2026-10-23 Fri");
//! let formats = JournalFormats { title, ..JournalFormats::default() };
//! assert_eq!(parse_journal_reference("Feb 1st, 2025", &formats), JournalDate::new(2025, 2, 1));
//! ```

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::garden::{DocumentKind, Garden, GardenDocument};
use crate::garden_settings::GardenSettings;
use crate::page_titles::title_property;

/// Logseq's title format when `:journal/page-title-format` is unset.
pub const DEFAULT_TITLE_FORMAT: &str = "MMM do, yyyy";
/// Logseq's file-name format when `:journal/file-name-format` is unset.
pub const DEFAULT_FILE_NAME_FORMAT: &str = "yyyy_MM_dd";
/// Formats Logseq accepts for a journal reference after the configured title format.
const REFERENCE_FORMATS: [&str; 3] = [DEFAULT_TITLE_FORMAT, "yyyy-MM-dd", DEFAULT_FILE_NAME_FORMAT];

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

/// A calendar date in the proleptic Gregorian calendar. Orders oldest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JournalDate {
    year: i32,
    month: u8,
    day: u8,
}

impl JournalDate {
    /// A real calendar date, or `None` for an impossible one such as 2025-02-30.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        ((1..=12).contains(&month) && (1..=days_in_month(year, month)).contains(&day))
            .then_some(Self { year, month, day })
    }

    /// The date `days` after 1970-01-01.
    pub fn from_unix_days(days: i64) -> Self {
        // Howard Hinnant's civil_from_days.
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Self { year, month, day }
    }

    /// Days since 1970-01-01, negative before it.
    pub fn unix_days(self) -> i64 {
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let yoe = year.rem_euclid(400);
        let month = i64::from(self.month);
        let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5
            + i64::from(self.day)
            - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// Weekday from Monday (0) to Sunday (6).
    pub fn weekday(self) -> u8 {
        // 1970-01-01 was a Thursday.
        (self.unix_days() + 3).rem_euclid(7) as u8
    }

    /// The year.
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month, 1 to 12.
    pub fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, starting at 1.
    pub fn day(self) -> u8 {
        self.day
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn ordinal(day: u8) -> &'static str {
    match (day % 10, day % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Year,
    Month { width: u8 },
    MonthName { long: bool },
    Day { width: u8 },
    DayOrdinal,
    Weekday { long: bool },
    Literal(String),
}

/// A date pattern in the subset of date-fns syntax that Logseq gardens use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateFormat {
    pattern: String,
    tokens: Vec<Token>,
}

/// A pattern token [`DateFormat`] does not support.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedToken(pub String);

impl fmt::Display for UnsupportedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported date token {:?}", self.0)
    }
}

impl Error for UnsupportedToken {}

impl DateFormat {
    /// Parse a pattern of `yyyy`, `MM`, `M`, `MMM`, `MMMM`, `dd`, `d`, `do`, `E`,
    /// `EEE`, `EEEE`, `'quoted'` literals and other non-letter characters.
    pub fn parse(pattern: &str) -> Result<Self, UnsupportedToken> {
        let chars: Vec<char> = pattern.chars().collect();
        let mut tokens = Vec::new();
        let mut literal = String::new();
        let mut index = 0;
        while index < chars.len() {
            let c = chars[index];
            if c == '\'' {
                index += 1;
                if chars.get(index) == Some(&'\'') {
                    literal.push('\'');
                    index += 1;
                    continue;
                }
                while index < chars.len() {
                    if chars[index] == '\'' {
                        if chars.get(index + 1) == Some(&'\'') {
                            literal.push('\'');
                            index += 2;
                            continue;
                        }
                        break;
                    }
                    literal.push(chars[index]);
                    index += 1;
                }
                index += 1;
                continue;
            }
            if !c.is_ascii_alphabetic() {
                literal.push(c);
                index += 1;
                continue;
            }
            let mut end = index;
            while end < chars.len() && chars[end] == c {
                end += 1;
            }
            let width = end - index;
            let token = match (c, width) {
                ('d', 1) if chars.get(end) == Some(&'o') => {
                    end += 1;
                    Token::DayOrdinal
                }
                ('y', 4) => Token::Year,
                ('M', 1 | 2) => Token::Month { width: width as u8 },
                ('M', 3 | 4) => Token::MonthName { long: width == 4 },
                ('d', 1 | 2) => Token::Day { width: width as u8 },
                // cljs-time writes `E` and `EEE` as the short name and has no `EE`.
                ('E', 1 | 3 | 4) => Token::Weekday { long: width == 4 },
                _ => {
                    return Err(UnsupportedToken(chars[index..end].iter().collect()));
                }
            };
            if !literal.is_empty() {
                tokens.push(Token::Literal(std::mem::take(&mut literal)));
            }
            tokens.push(token);
            index = end;
        }
        if !literal.is_empty() {
            tokens.push(Token::Literal(literal));
        }
        Ok(Self {
            pattern: pattern.to_owned(),
            tokens,
        })
    }

    /// The pattern as written.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Write a date in this format.
    pub fn format(&self, date: JournalDate) -> String {
        let mut text = String::new();
        for token in &self.tokens {
            match token {
                Token::Year => text.push_str(&format!("{:04}", date.year)),
                Token::Month { width } => {
                    text.push_str(&format!("{:0width$}", date.month, width = *width as usize))
                }
                Token::MonthName { long } => {
                    let name = MONTHS[usize::from(date.month - 1)];
                    text.push_str(if *long { name } else { &name[..3] });
                }
                Token::Day { width } => {
                    text.push_str(&format!("{:0width$}", date.day, width = *width as usize))
                }
                Token::DayOrdinal => text.push_str(&format!("{}{}", date.day, ordinal(date.day))),
                Token::Weekday { long } => {
                    let name = WEEKDAYS[usize::from(date.weekday())];
                    text.push_str(if *long { name } else { &name[..3] });
                }
                Token::Literal(literal) => text.push_str(literal),
            }
        }
        text
    }

    /// Read a whole string in this format. Month and weekday names match without
    /// regard to case, and a weekday may be its full or short name whatever the
    /// pattern's width, as in Logseq. Impossible dates and contradicting weekdays
    /// give `None`.
    pub fn read(&self, text: &str) -> Option<JournalDate> {
        let mut rest = text;
        let (mut year, mut month, mut day, mut weekday) = (None, None, None, None);
        for token in &self.tokens {
            match token {
                Token::Year => {
                    let (value, tail) = digits(rest, 4, 4)?;
                    year = Some(value as i32);
                    rest = tail;
                }
                Token::Month { width } => {
                    let (value, tail) = digits(rest, usize::from(*width), 2)?;
                    month = Some(value as u8);
                    rest = tail;
                }
                Token::Day { width } => {
                    let (value, tail) = digits(rest, usize::from(*width), 2)?;
                    day = Some(value as u8);
                    rest = tail;
                }
                Token::DayOrdinal => {
                    let (value, tail) = digits(rest, 1, 2)?;
                    day = Some(value as u8);
                    rest = ["st", "nd", "rd", "th"]
                        .iter()
                        .find_map(|suffix| strip_prefix_ignore_case(tail, suffix))
                        .unwrap_or(tail);
                }
                Token::MonthName { long } => {
                    let (index, tail) = name(rest, &MONTHS, *long)?;
                    month = Some(index as u8 + 1);
                    rest = tail;
                }
                Token::Weekday { .. } => {
                    let (index, tail) =
                        name(rest, &WEEKDAYS, true).or_else(|| name(rest, &WEEKDAYS, false))?;
                    weekday = Some(index as u8);
                    rest = tail;
                }
                Token::Literal(literal) => rest = rest.strip_prefix(literal.as_str())?,
            }
        }
        if !rest.is_empty() {
            return None;
        }
        let date = JournalDate::new(year?, month?, day?)?;
        weekday
            .is_none_or(|weekday| weekday == date.weekday())
            .then_some(date)
    }
}

fn digits(text: &str, min: usize, max: usize) -> Option<(u32, &str)> {
    let count = text
        .bytes()
        .take(max)
        .take_while(u8::is_ascii_digit)
        .count();
    if count < min {
        return None;
    }
    Some((text[..count].parse().ok()?, &text[count..]))
}

fn strip_prefix_ignore_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let head = text.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| &text[prefix.len()..])
}

fn name<'a>(text: &'a str, names: &[&str], long: bool) -> Option<(usize, &'a str)> {
    names.iter().enumerate().find_map(|(index, name)| {
        let name = if long { name } else { &name[..3] };
        strip_prefix_ignore_case(text, name).map(|rest| (index, rest))
    })
}

/// The title and file-name formats a garden uses for journals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalFormats {
    /// `:journal/page-title-format`
    pub title: DateFormat,
    /// `:journal/file-name-format`
    pub file_name: DateFormat,
}

impl Default for JournalFormats {
    fn default() -> Self {
        Self {
            title: DateFormat::parse(DEFAULT_TITLE_FORMAT).expect("default title format"),
            file_name: DateFormat::parse(DEFAULT_FILE_NAME_FORMAT)
                .expect("default file-name format"),
        }
    }
}

impl JournalFormats {
    /// Formats from garden settings, which already replaced invalid patterns with defaults.
    pub fn from_settings(settings: &GardenSettings) -> Self {
        let defaults = Self::default();
        Self {
            title: DateFormat::parse(&settings.journal_title_format).unwrap_or(defaults.title),
            file_name: DateFormat::parse(&settings.journal_file_name_format)
                .unwrap_or(defaults.file_name),
        }
    }
}

/// A journal's date and display title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalIdentity {
    /// The date from the file name.
    pub date: JournalDate,
    /// The leading `title::` property, or the date in the title format.
    pub title: String,
}

/// A journal file whose name is not a date in the file-name format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalNameError {
    /// Source path relative to the garden root.
    pub path: PathBuf,
}

impl fmt::Display for JournalNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: journal file name is not a date in the journal file-name format",
            self.path.display()
        )
    }
}

impl Error for JournalNameError {}

/// Derive a journal's date from its file stem and its title, without filesystem access.
///
/// Documents outside `journals/` return an error, as do stems that are not real dates.
pub fn journal_identity(
    document: &GardenDocument,
    formats: &JournalFormats,
) -> Result<JournalIdentity, JournalNameError> {
    let error = || JournalNameError {
        path: document.relative_path.clone(),
    };
    if document.kind != DocumentKind::Journal {
        return Err(error());
    }
    let date = Path::new(&document.relative_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| formats.file_name.read(stem))
        .ok_or_else(error)?;
    let title = title_property(document)
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| formats.title.format(date));
    Ok(JournalIdentity { date, title })
}

/// Parse a page reference as a journal date in the configured title format,
/// `MMM do, yyyy`, `yyyy-MM-dd` or `yyyy_MM_dd`, tried in that order.
pub fn parse_journal_reference(name: &str, formats: &JournalFormats) -> Option<JournalDate> {
    let name = name.trim();
    static FALLBACKS: LazyLock<Vec<DateFormat>> = LazyLock::new(|| {
        REFERENCE_FORMATS
            .iter()
            .map(|pattern| DateFormat::parse(pattern).expect("reference format"))
            .collect()
    });
    formats
        .title
        .read(name)
        .or_else(|| FALLBACKS.iter().find_map(|format| format.read(name)))
}

/// A journal document with its identity.
#[derive(Debug, Clone)]
pub struct Journal<'a> {
    /// The journal's date and title.
    pub identity: JournalIdentity,
    /// The loaded source.
    pub document: &'a GardenDocument,
}

/// Every journal in a garden, newest first, and an error for each unparseable name.
pub fn journal_list<'a>(
    garden: &'a Garden,
    formats: &JournalFormats,
) -> (Vec<Journal<'a>>, Vec<JournalNameError>) {
    let mut journals = Vec::new();
    let mut errors = Vec::new();
    for document in garden
        .documents
        .iter()
        .filter(|document| document.kind == DocumentKind::Journal)
    {
        match journal_identity(document, formats) {
            Ok(identity) => journals.push(Journal { identity, document }),
            Err(error) => errors.push(error),
        }
    }
    journals.sort_by(|left, right| {
        right.identity.date.cmp(&left.identity.date).then_with(|| {
            left.document
                .relative_path
                .cmp(&right.document.relative_path)
        })
    });
    (journals, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u8, day: u8) -> JournalDate {
        JournalDate::new(year, month, day).unwrap()
    }

    fn format(pattern: &str) -> DateFormat {
        DateFormat::parse(pattern).unwrap()
    }

    #[test]
    fn writes_encode_garden_and_default_titles() {
        assert_eq!(
            format("yyyy-MM-dd EEE").format(date(2026, 10, 23)),
            "2026-10-23 Fri"
        );
        let defaults = JournalFormats::default();
        assert_eq!(defaults.title.format(date(2026, 10, 23)), "Oct 23rd, 2026");
        assert_eq!(
            defaults.file_name.read("2026_10_23"),
            Some(date(2026, 10, 23))
        );
        assert_eq!(
            format("EEEE, MMMM d 'of' yyyy").format(date(2025, 2, 1)),
            "Saturday, February 1 of 2025"
        );
        for (day, text) in [
            (1, "1st"),
            (2, "2nd"),
            (3, "3rd"),
            (4, "4th"),
            (11, "11th"),
            (12, "12th"),
            (13, "13th"),
            (21, "21st"),
            (22, "22nd"),
            (31, "31st"),
        ] {
            assert_eq!(format("do").format(date(2025, 1, day)), text);
        }
    }

    #[test]
    fn reads_dates_strictly() {
        let encode = format("yyyy-MM-dd EEE");
        assert_eq!(encode.read("2026-10-23 fri"), Some(date(2026, 10, 23)));
        assert_eq!(encode.read("2026-10-23 Mon"), None);
        assert_eq!(encode.read("2026-10-23 Fri "), None);
        assert_eq!(format("yyyy_MM_dd").read("2025_02_30"), None);
        assert_eq!(
            format("yyyy_MM_dd").read("2024_02_29"),
            Some(date(2024, 2, 29))
        );
        assert_eq!(
            format("MMM do, yyyy").read("feb 1st, 2025"),
            Some(date(2025, 2, 1))
        );
        assert_eq!(
            format("MMM do, yyyy").read("Feb 22, 2025"),
            Some(date(2025, 2, 22))
        );
        assert_eq!(
            format("MMMM d, yyyy").read("February 1, 2025"),
            Some(date(2025, 2, 1))
        );
    }

    #[test]
    fn writes_and_reads_weekday_tokens_as_logseq_does() {
        // cljs-time writes `E` and `EEE` as the short name and `EEEE` as the full name.
        let friday = date(2026, 10, 23);
        for (pattern, text) in [
            ("E, MM/dd/yyyy", "Fri, 10/23/2026"),
            ("EEE, dd.MM.yyyy", "Fri, 23.10.2026"),
            ("EEEE, yyyy/MM/dd", "Friday, 2026/10/23"),
            ("yyyy-MM-dd E", "2026-10-23 Fri"),
        ] {
            let format = format(pattern);
            assert_eq!(format.format(friday), text, "{pattern}");
            assert_eq!(format.read(text), Some(friday), "{pattern}");
        }
        // Reading accepts either name length, without regard to case.
        assert_eq!(
            format("E, MM/dd/yyyy").read("friday, 10/23/2026"),
            Some(friday)
        );
        assert_eq!(
            format("EEEE, MM/dd/yyyy").read("FRI, 10/23/2026"),
            Some(friday)
        );
        assert_eq!(format("E, MM/dd/yyyy").read("Mon, 10/23/2026"), None);
        for days in 20_000..20_007 {
            let day = JournalDate::from_unix_days(days);
            for pattern in ["E, MM/dd/yyyy", "EEE, MM/dd/yyyy", "EEEE, MM/dd/yyyy"] {
                let format = format(pattern);
                assert_eq!(format.read(&format.format(day)), Some(day), "{pattern}");
            }
        }
    }

    #[test]
    fn rejects_unsupported_tokens() {
        assert_eq!(
            DateFormat::parse("yyyy QQQ"),
            Err(UnsupportedToken("QQQ".into()))
        );
        assert!(DateFormat::parse("yy-MM").is_err());
        // cljs-time has no `EE` or `EEEEE` pattern, so Logseq cannot use them.
        assert_eq!(
            DateFormat::parse("EE yyyy"),
            Err(UnsupportedToken("EE".into()))
        );
        assert!(DateFormat::parse("EEEEE yyyy").is_err());
    }

    #[test]
    fn computes_weekdays_and_unix_days() {
        assert_eq!(date(1970, 1, 1).weekday(), 3);
        assert_eq!(date(2026, 10, 23).weekday(), 4);
        assert_eq!(date(1932, 1, 24).weekday(), 6);
        for days in [-800_000, -1, 0, 1, 20_384, 60_000] {
            assert_eq!(JournalDate::from_unix_days(days).unix_days(), days);
        }
        assert_eq!(JournalDate::from_unix_days(20_386), date(2025, 10, 25));
    }

    fn journal(path: &str, source: &str) -> GardenDocument {
        GardenDocument {
            relative_path: path.into(),
            kind: DocumentKind::Journal,
            source: source.to_owned(),
            parsed: lsdoc::parse_to_projection(source),
        }
    }

    #[test]
    fn derives_identity_and_lists_newest_first() {
        let formats = JournalFormats {
            title: format("yyyy-MM-dd EEE"),
            ..JournalFormats::default()
        };
        let identity = journal_identity(&journal("journals/2026_10_23.md", "- Filed\n"), &formats);
        assert_eq!(
            identity,
            Ok(JournalIdentity {
                date: date(2026, 10, 23),
                title: "2026-10-23 Fri".into()
            })
        );
        let titled = journal_identity(
            &journal("journals/2026_10_23.md", "title:: Launch day\n\n- text\n"),
            &formats,
        )
        .unwrap();
        assert_eq!(
            (titled.date, titled.title.as_str()),
            (date(2026, 10, 23), "Launch day")
        );
        let impossible = journal("journals/2025_02_30.md", "- x\n");
        assert_eq!(
            journal_identity(&impossible, &formats).unwrap_err().path,
            PathBuf::from("journals/2025_02_30.md")
        );
        let garden = Garden {
            documents: ["2026_10_21", "2026_10_23", "notes", "2026_10_22"]
                .iter()
                .map(|stem| journal(&format!("journals/{stem}.md"), "- x\n"))
                .collect(),
        };
        let (journals, errors) = journal_list(&garden, &formats);
        let titles: Vec<_> = journals.iter().map(|j| j.identity.title.as_str()).collect();
        assert_eq!(
            titles,
            ["2026-10-23 Fri", "2026-10-22 Thu", "2026-10-21 Wed"]
        );
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn parses_references_in_logseq_formats() {
        let formats = JournalFormats {
            title: format("yyyy-MM-dd EEE"),
            ..JournalFormats::default()
        };
        for (name, expected) in [
            ("Feb 1st, 2025", Some(date(2025, 2, 1))),
            ("2026-10-23 fri", Some(date(2026, 10, 23))),
            ("2026-10-23", Some(date(2026, 10, 23))),
            ("2026_10_23", Some(date(2026, 10, 23))),
            ("2026-10-23 Mon", None),
            ("tmux", None),
        ] {
            assert_eq!(parse_journal_reference(name, &formats), expected, "{name}");
        }
    }
}
