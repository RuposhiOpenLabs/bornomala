/// Converts numerical currency amounts into formal Bangla and English words.

const BANGLA_ONES: &[&str] = &[
    "", "এক", "দুই", "তিন", "চার", "পাঁচ", "ছয়", "সাত", "আট", "নয়",
    "দশ", "এগারো", "বারো", "তেরো", "চৌদ্দ", "পনেরো", "ষোলো", "সতেরো", "আঠারো", "উনিশ",
    "বিশ", "একুশ", "বাইশ", "তেইশ", "চব্বিশ", "পঁচিশ", "ছাব্বিশ", "সাতাশ", "আটাশ", "ঊনত্রিশ",
    "ত্রিশ", "একত্রিশ", "বত্রিশ", "তেত্রিশ", "চৌত্রিশ", "পঁয়ত্রিশ", "ছত্রিশ", "সাঁইত্রিশ", "আটত্রিশ", "ঊনচল্লিশ",
    "চল্লিশ", "একচল্লিশ", "বিয়াল্লিশ", "তেতাল্লিশ", "চুয়াল্লিশ", "পঁয়তাল্লিশ", "ছেচল্লিশ", "সাতচল্লিশ", "আটচল্লিশ", "ঊনপঞ্চাশ",
    "পঞ্চাশ", "একান্ন", "বায়ান্ন", "তিপ্পান্ন", "চুয়ান্ন", "পঞ্চান্ন", "ছাপ্পান্ন", "সাতান্ন", "আটান্ন", "ঊনষাট",
    "ষাট", "একষট্টি", "বাষট্টি", "তেষট্টি", "চৌষট্টি", "পঁয়ষট্টি", "ছেষট্টি", "সাতষট্টি", "আটষট্টি", "ঊনসত্তর",
    "সত্তর", "একাত্তর", "বাহাত্তর", "তিয়াত্তর", "চুয়াত্তর", "পঁচাত্তর", "ছিয়াত্তর", "সাতাত্তর", "আটাত্তর", "ঊনআশি",
    "আশি", "একাশি", "বিরাশি", "তিরাশি", "চুরাশি", "পঁচাশি", "ছিয়াশি", "সাতাশি", "আটাশি", "ঊননব্বই",
    "নব্বই", "একানব্বই", "বানব্বই", "তিরানব্বই", "চুরানব্বই", "পঁচানব্বই", "ছিয়ানব্বই", "সাতানব্বই", "আটানব্বই", "নিরানব্বই",
];

const ENGLISH_ONES: &[&str] = &[
    "", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine",
    "Ten", "Eleven", "Twelve", "Thirteen", "Fourteen", "Fifteen", "Sixteen", "Seventeen", "Eighteen", "Nineteen",
];

const ENGLISH_TENS: &[&str] = &[
    "", "", "Twenty", "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety",
];

/// Converts an amount to official Bangla words with Taka and Paisa.
pub fn amount_to_bangla_words(amount: f64) -> String {
    if amount < 0.0 {
        return format!("ঋণাত্মক {}", amount_to_bangla_words(-amount));
    }

    let taka = amount.floor() as u64;
    let paisa = ((amount - taka as f64) * 100.0).round() as u64;

    let mut parts = Vec::new();

    if taka == 0 {
        parts.push("শূন্য টাকা".to_string());
    } else {
        let taka_words = integer_to_bangla_words(taka);
        parts.push(format!("{} টাকা", taka_words));
    }

    if paisa > 0 && paisa < 100 {
        let paisa_words = BANGLA_ONES[paisa as usize];
        parts.push(format!("{} পয়সা", paisa_words));
    }

    format!("{} মাত্র", parts.join(" "))
}

fn integer_to_bangla_words(n: u64) -> String {
    if n == 0 {
        return "শূন্য".to_string();
    }

    let mut result = Vec::new();

    let crore = n / 10_000_000;
    let rem_crore = n % 10_000_000;

    let lakh = rem_crore / 100_000;
    let rem_lakh = rem_crore % 100_000;

    let thousand = rem_lakh / 1_000;
    let rem_thousand = rem_lakh % 1_000;

    let hundred = rem_thousand / 100;
    let rest = rem_thousand % 100;

    if crore > 0 {
        result.push(format!("{} কোটি", integer_to_bangla_words(crore)));
    }
    if lakh > 0 {
        result.push(format!("{} লাখ", BANGLA_ONES[lakh as usize]));
    }
    if thousand > 0 {
        result.push(format!("{} হাজার", BANGLA_ONES[thousand as usize]));
    }
    if hundred > 0 {
        result.push(format!("{} শত", BANGLA_ONES[hundred as usize]));
    }
    if rest > 0 {
        result.push(BANGLA_ONES[rest as usize].to_string());
    }

    result.join(" ")
}

/// Converts an amount to English words with Taka and Paisa.
pub fn amount_to_english_words(amount: f64) -> String {
    if amount < 0.0 {
        return format!("Minus {}", amount_to_english_words(-amount));
    }

    let taka = amount.floor() as u64;
    let paisa = ((amount - taka as f64) * 100.0).round() as u64;

    let mut parts = Vec::new();

    if taka == 0 {
        parts.push("Zero Taka".to_string());
    } else {
        let taka_words = integer_to_english_words(taka);
        parts.push(format!("{} Taka", taka_words));
    }

    if paisa > 0 && paisa < 100 {
        let paisa_words = number_under_100_to_english(paisa);
        parts.push(format!("and {} Paisa", paisa_words));
    }

    format!("{} Only", parts.join(" "))
}

fn integer_to_english_words(n: u64) -> String {
    if n == 0 {
        return "Zero".to_string();
    }

    let mut parts = Vec::new();

    let crore = n / 10_000_000;
    let rem_crore = n % 10_000_000;

    let lakh = rem_crore / 100_000;
    let rem_lakh = rem_crore % 100_000;

    let thousand = rem_lakh / 1_000;
    let rem_thousand = rem_lakh % 1_000;

    let hundred = rem_thousand / 100;
    let rest = rem_thousand % 100;

    if crore > 0 {
        parts.push(format!("{} Crore", integer_to_english_words(crore)));
    }
    if lakh > 0 {
        parts.push(format!("{} Lakh", number_under_100_to_english(lakh)));
    }
    if thousand > 0 {
        parts.push(format!("{} Thousand", number_under_100_to_english(thousand)));
    }
    if hundred > 0 {
        parts.push(format!("{} Hundred", ENGLISH_ONES[hundred as usize]));
    }
    if rest > 0 {
        parts.push(number_under_100_to_english(rest));
    }

    parts.join(" ")
}

fn number_under_100_to_english(n: u64) -> String {
    if n < 20 {
        ENGLISH_ONES[n as usize].to_string()
    } else {
        let tens = n / 10;
        let units = n % 10;
        if units == 0 {
            ENGLISH_TENS[tens as usize].to_string()
        } else {
            format!("{}-{}", ENGLISH_TENS[tens as usize], ENGLISH_ONES[units as usize])
        }
    }
}
