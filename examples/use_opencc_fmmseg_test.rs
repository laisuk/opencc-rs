use opencc_rs::Opencc;

fn main() {
    // ---------------------------------------------------------------------
    // Sample UTF-8 input (same spirit as C / C++ demos)
    // ---------------------------------------------------------------------
    let input_text = "意大利邻国法兰西罗浮宫里收藏的“蒙娜丽莎的微笑”画像是旷世之作。";

    println!("Text:");
    println!("{}", input_text);
    println!();

    // ---------------------------------------------------------------------
    // Create OpenCC instance
    // ---------------------------------------------------------------------
    let converter = Opencc::new();

    // Detect script
    let input_code = converter.zho_check(input_text);
    println!("Text Code: {}", input_code);

    // ---------------------------------------------------------------------
    // Test 1: Legacy string-based config (convert)
    // ---------------------------------------------------------------------
    let config_str = "s2twp";
    let punct = true;

    println!();
    println!(
        "== Test 1: convert(config = \"{}\", punctuation = {}) ==",
        config_str, punct
    );

    let output1 = converter.convert_with_punctuation(input_text, config_str);
    println!("Converted:");
    println!("{}", output1);
    println!("Converted Code: {}", converter.zho_check(&output1));

    // ---------------------------------------------------------------------
    // Summary
    // ---------------------------------------------------------------------
    println!();
    println!("All tests completed.");
}
