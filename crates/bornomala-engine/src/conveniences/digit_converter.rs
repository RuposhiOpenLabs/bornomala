/// Converts between English (ASCII) digits and Bangla numerals.

pub fn convert_digits(text: &str, to_bangla: bool) -> String {
    text.chars()
        .map(|c| {
            if to_bangla {
                match c {
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
                }
            } else {
                match c {
                    '০' => '0',
                    '১' => '1',
                    '২' => '2',
                    '৩' => '3',
                    '৪' => '4',
                    '৫' => '5',
                    '৬' => '6',
                    '৭' => '7',
                    '৮' => '8',
                    '৯' => '9',
                    other => other,
                }
            }
        })
        .collect()
}
