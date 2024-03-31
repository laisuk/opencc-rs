use opencc_rs::{find_max_utf8_length, format_thousand, Opencc};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_punctuation_test() {
        let input = "他説：“你好！歡樂‘龍龍’”";
        let expected_output = "他説：「你好！歡樂『龍龍』」";
        let opencc = Opencc::new();
        let actual_output = opencc.convert_with_punctuation(input, "s2t");
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn find_max_utf8_length_test() {
        let input = "你好，世界！";
        let expected_output = 9;
        let actual_output = find_max_utf8_length(input, 10);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn zho_check_test() {
        let input = "他説：「你好！歡樂『龍龍』」";
        let expected_output = 1;
        let opencc = Opencc::new();
        let actual_output = opencc.zho_check(input);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn convert_to_buffer_test() {
        let input = "Hello, World!";
        let expected_output = "Hello, World!";
        let opencc = Opencc::new();
        let actual_output = opencc.convert_to_buffer(input, "s2t");
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn set_conversion_test() {
        let opencc = Opencc::new();
        let text = opencc.convert("龙马精神", "s2t");
        let config = "t2s";
        assert_eq!(opencc.convert(text.as_str(), config), "龙马精神");
    }

    #[test]
    fn convert_test() {
        let input = "龙马精神!";
        let expected_output = "龍馬精神!";
        let opencc = Opencc::new();
        let actual_output = opencc.convert(input, "s2t");
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn convert_with_punctuation_test() {
        let input = "“龙马精神!”";
        let expected_output = "「龍馬精神!」";
        let opencc = Opencc::new();
        let actual_output = opencc.convert_with_punctuation(input, "s2t");
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn format_thousand_test() {
        let input = 1234567890;
        let expected_output = "1,234,567,890";
        let actual_output = format_thousand(input);
        assert_eq!(actual_output, expected_output);
    }
}
