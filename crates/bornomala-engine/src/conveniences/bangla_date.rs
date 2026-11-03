/// Revised Bangla Calendar (Bangladesh Official Standard, 2019 Revision).

pub const BANGLA_MONTH_NAMES: &[&str] = &[
    "বৈশাখ", "জ্যৈষ্ঠ", "আষাঢ়", "শ্রাবণ", "ভাদ্র", "আশ্বিন",
    "কার্তিক", "অগ্রহায়ণ", "পৌষ", "মাঘ", "ফাল্গুন", "চৈত্র",
];

pub const BANGLA_DAY_NAMES: &[&str] = &[
    "রবিবার", "সোমবার", "মঙ্গলবার", "বুধবার", "বৃহস্পতিবার", "শুক্রবার", "শনিবার",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BanglaDate {
    pub year: i32,
    pub month: u32, // 1 to 12
    pub day: u32,   // 1 to 31
    pub month_name: &'static str,
}

impl BanglaDate {
    /// Formats the Bangla date in Bangla numerals and month name.
    /// e.g. "১ বৈশাখ ১৪৩১"
    pub fn format_bangla(&self) -> String {
        format!(
            "{} {} {}",
            to_bangla_digits(&self.day.to_string()),
            self.month_name,
            to_bangla_digits(&self.year.to_string())
        )
    }
}

pub fn is_gregorian_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Converts a Gregorian Date (year, month 1..=12, day 1..=31) to Revised Bangla Date.
pub fn gregorian_to_bangla(year: i32, month: u32, day: u32) -> BanglaDate {
    let leap = is_gregorian_leap_year(year);

    // Cumulative days from April 14 of current Gregorian year
    // Days in Gregorian months
    let g_days = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];

    // Day of year for the input date (1-indexed)
    let mut day_of_year = day;
    for m in 0..(month as usize - 1) {
        day_of_year += g_days[m];
    }

    // April 14 day of year
    let april_14 = 31 + g_days[1] + 31 + 14;

    let (b_year, days_since_baishakh_1) = if day_of_year >= april_14 {
        (year - 593, day_of_year - april_14)
    } else {
        let prev_leap = is_gregorian_leap_year(year - 1);
        let prev_days_in_year = if prev_leap { 366 } else { 365 };
        let prev_april_14 = 31 + (if prev_leap { 29 } else { 28 }) + 31 + 14;
        let days_from_prev_april_14 = (prev_days_in_year - prev_april_14) + day_of_year;
        (year - 594, days_from_prev_april_14)
    };

    // Month lengths in revised calendar (2019 Bangladesh Standard):
    // Baishakh to Ashwin (months 1-6): 31 days each (186 days total)
    // Kartik to Magh (months 7-10): 30 days each (120 days total)
    // Falgun (month 11): 29 days (30 in Gregorian leap year)
    // Chaitra (month 12): 30 days
    let falgun_len = if leap { 30 } else { 29 };
    let b_month_lens = [31, 31, 31, 31, 31, 31, 30, 30, 30, 30, falgun_len, 30];

    let mut rem_days = days_since_baishakh_1;
    let mut b_month = 1;

    for (m_idx, &len) in b_month_lens.iter().enumerate() {
        if rem_days < len {
            b_month = (m_idx + 1) as u32;
            break;
        }
        rem_days -= len;
    }

    let b_day = rem_days + 1;
    let month_name = BANGLA_MONTH_NAMES[(b_month - 1) as usize];

    BanglaDate {
        year: b_year,
        month: b_month,
        day: b_day,
        month_name,
    }
}

/// Converts ASCII digits (`0-9`) to Bangla digits (`০-৯`).
pub fn to_bangla_digits(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '0' => '০',
            '1' => '১',
            '2' => '২',
            '3' => '৩',
            '4' => '৪',
            '5' => '৫',
            '6' => '৬',
            '7' => '৭',
            '8' => '৮',
            '9' => '৯',
            other => other,
        })
        .collect()
}
