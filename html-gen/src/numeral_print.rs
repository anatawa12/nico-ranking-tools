use num_format::{Locale, ToFormattedString};

pub fn numeral_to_string(num: u64) -> String {
    if num < 1_0000 {
        // 1万未満
        num.to_formatted_string(&Locale::ja)
    } else if num < 1_0000_0000 {
        // 1億未満
        format!("{}万", (num / 1000) as f64 / 10.0)
    } else {
        unimplemented!("億")
    }
}