use std::borrow::Cow;
use lazy_static::lazy_static;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

use libloading::os::windows::Symbol as RawSymbol;
use libloading::{Library, Symbol};
use regex::Regex;

// Define the type for opencc_t
type OpenccT = *mut c_void;
type OpenccOpenFunc = unsafe extern "C" fn(config_file_name: *const c_char) -> OpenccT;
type OpenccCloseFunc = unsafe extern "C" fn(opencc: OpenccT) -> c_int;
type OpenccConvertUtf8Func =
    unsafe extern "C" fn(opencc: OpenccT, input: *const c_char, length: usize) -> *mut c_char;
type OpenccConvertUtf8FreeFunc = unsafe extern "C" fn(str: *mut c_char);
type OpenccConvertUtf8ToBufferFunc = unsafe extern "C" fn(
    opencc: OpenccT,
    input: *const c_char,
    length: usize,
    output: *mut c_char,
) -> isize;
type OpenccErrorFunc = unsafe extern "C" fn() -> *const c_char;

lazy_static! {
    static ref STRIP_REGEX: Regex = Regex::new(r"[!-/:-@\[-`{-~\t\n\v\f\r 0-9A-Za-z_]").unwrap();
}

const CONFIG_VECTOR: [&str; 14] = [
    "s2t", "t2s", "s2tw", "tw2s", "s2twp", "tw2sp", "tw2t", "t2tw", "s2hk", "hk2s", "hk2t", "t2hk",
    "t2jp", "jp2t",
];
pub struct Opencc {
    pub lib: Library,
    opencc_open: RawSymbol<OpenccOpenFunc>,
    opencc_close: RawSymbol<OpenccCloseFunc>,
    opencc_convert_utf8: RawSymbol<OpenccConvertUtf8Func>,
    opencc_convert_utf8_free: RawSymbol<OpenccConvertUtf8FreeFunc>,
    opencc_convert_utf8_to_buffer: RawSymbol<OpenccConvertUtf8ToBufferFunc>,
    opencc_error: RawSymbol<OpenccErrorFunc>,
    config_file_path: String,
}

impl Opencc {
    // #[allow(unused_variables)]
    pub fn new() -> Self {
        let lib = unsafe { Library::new("lib/opencc") }.expect("Failed to load DLL");
        unsafe {
            let opencc_open: Symbol<OpenccOpenFunc> =
                lib.get(b"opencc_open").expect("Failed to load opencc_open");
            let opencc_open = opencc_open.into_raw();
            let opencc_close: Symbol<OpenccCloseFunc> = lib
                .get(b"opencc_close")
                .expect("Failed to load opencc_close");
            let opencc_close = opencc_close.into_raw();
            let opencc_convert_utf8: Symbol<OpenccConvertUtf8Func> = lib
                .get(b"opencc_convert_utf8")
                .expect("Failed to load opencc_convert_utf8");
            let opencc_convert_utf8 = opencc_convert_utf8.into_raw();
            let opencc_convert_utf8_free: Symbol<OpenccConvertUtf8FreeFunc> = lib
                .get(b"opencc_convert_utf8_free")
                .expect("Failed to load opencc_convert_utf8_free");
            let opencc_convert_utf8_free = opencc_convert_utf8_free.into_raw();
            let opencc_convert_utf8_to_buffer: Symbol<OpenccConvertUtf8ToBufferFunc> = lib
                .get(b"opencc_convert_utf8_to_buffer")
                .expect("Failed to load opencc_convert_utf8_to_buffer");
            let opencc_convert_utf8_to_buffer = opencc_convert_utf8_to_buffer.into_raw();
            let opencc_error: Symbol<OpenccErrorFunc> = lib
                .get(b"opencc_error")
                .expect("Failed to load opencc_error");
            let opencc_error = opencc_error.into_raw();
            let config_file_path = "lib/opencc".to_string();

            Opencc {
                lib,
                opencc_open,
                opencc_close,
                opencc_convert_utf8,
                opencc_convert_utf8_free,
                opencc_convert_utf8_to_buffer,
                opencc_error,
                config_file_path,
            }
        } //unsafe
    }

    pub fn convert(&self, input: &str, config: &str) -> String {
        let mut config = config;
        if !CONFIG_VECTOR.contains(&config) {
            config = "s2t";
        }
        let converted_str = self.convert_by(input, config);
        converted_str
    }

    fn convert_by(&self, input: &str, config: &str) -> String {
        let input_str = CString::new(input).unwrap();
        let config = CString::new(format!("{}/{}.json", &self.config_file_path, config)).unwrap();
        let mut free_open = Vec::new();
        let mut free_char = Vec::new();
        unsafe {
            let opencc = (self.opencc_open)(config.as_ptr());
            free_open.push(opencc);
            let output =
                (self.opencc_convert_utf8)(opencc, input_str.as_ptr(), input_str.as_bytes().len());
            if output.is_null() {
                return format!(
                    "Error convert UTF-8 string: {}",
                    CStr::from_ptr((self.opencc_error)()).to_str().unwrap()
                );
            }
            free_char.push(output);
            let converted_str = CStr::from_ptr(output).to_string_lossy().into_owned();
            for ptr in free_char {
                (self.opencc_convert_utf8_free)(ptr);
            }
            for ptr in free_open {
                (self.opencc_close)(ptr);
            }
            converted_str
        }
    }

    pub fn convert_to_buffer(&self, input: &str, config: &str) -> String {
        let mut config = config;
        if !CONFIG_VECTOR.contains(&config) {
            config = "s2t";
        }
        let converted_str = self.convert_by_buffer(input, config);
        converted_str
    }

    fn convert_by_buffer(&self, input: &str, config: &str) -> String {
        let input_str = CString::new(input).unwrap();
        let input_str_length = input_str.as_bytes().len();
        let mut output_buffer: Vec<c_char> = vec![0; input_str_length + 1];
        let config = CString::new(format!("{}/{}.json", &self.config_file_path, config)).unwrap();
        let mut free_open = Vec::new();
        unsafe {
            let opencc = (self.opencc_open)(config.as_ptr());
            free_open.push(opencc);
            let converted_length = (self.opencc_convert_utf8_to_buffer)(
                opencc,
                input_str.as_ptr(),
                input_str_length,
                output_buffer.as_mut_ptr(),
            );
            for ptr in free_open {
                (self.opencc_close)(ptr);
            }
            if converted_length == -1 {
                format!(
                    "Error convert UTF-8 string: {}",
                    CStr::from_ptr((self.opencc_error)()).to_str().unwrap()
                )
            } else {
                CStr::from_ptr(output_buffer.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            }
        }
    }

    pub fn convert_with_punctuation(&self, input: &str, config: &str) -> String {
        let converted_str = self.convert_by(input, config);
        Self::convert_punctuation_cow(&converted_str, config).into_owned()
    }

    pub fn zho_check(&self, input: &str) -> i8 {
        if input.is_empty() {
            return 0;
        }
        let _strip_text = STRIP_REGEX.replace_all(input, "");
        let max_bytes = find_max_utf8_length(_strip_text.as_ref(), 200);
        let strip_text = &_strip_text[..max_bytes];

        let code;
        if strip_text != &self.convert_by(strip_text, "t2s") {
            code = 1;
        } else {
            if strip_text != &self.convert_by(strip_text, "s2t") {
                code = 2;
            } else {
                code = 0;
            }
        }
        code
    }

    //     fn convert_punctuation(sv: &str, config: &str) -> String {
    //         let mut s2t_punctuation_chars: HashMap<&str, &str> = HashMap::new();
    //         s2t_punctuation_chars.insert("“", "「");
    //         s2t_punctuation_chars.insert("”", "」");
    //         s2t_punctuation_chars.insert("‘", "『");
    //         s2t_punctuation_chars.insert("’", "』");
    //
    //         let output_text;
    //
    //         if config.starts_with('s') || !config.contains('s') {
    //             let s2t_pattern = s2t_punctuation_chars.keys().cloned().collect::<String>();
    //             let s2t_regex = Regex::new(&format!("[{}]", s2t_pattern)).unwrap();
    //             output_text = s2t_regex
    //                 .replace_all(sv, |caps: &regex::Captures| {
    //                     s2t_punctuation_chars[caps.get(0).unwrap().as_str()]
    //                 })
    //                 .into_owned();
    //         } else {
    //             let mut t2s_punctuation_chars: HashMap<&str, &str> = HashMap::new();
    //             for (key, value) in s2t_punctuation_chars.iter() {
    //                 t2s_punctuation_chars.insert(value, key);
    //             }
    //             let t2s_pattern = t2s_punctuation_chars.keys().cloned().collect::<String>();
    //             let t2s_regex = Regex::new(&format!("[{}]", t2s_pattern)).unwrap();
    //             output_text = t2s_regex
    //                 .replace_all(sv, |caps: &regex::Captures| {
    //                     t2s_punctuation_chars[caps.get(0).unwrap().as_str()]
    //                 })
    //                 .into_owned();
    //         }
    //         output_text
    //     }
    #[inline]
    #[allow(dead_code)]
    fn convert_punctuation(sv: &str, config: &str) -> String {
        // Your old rule:
        // - if config starts with 's' OR config doesn't contain 's' -> do S2T punct
        // - else -> do T2S punct
        let s2t = config.starts_with('s') || !config.contains('s');

        // Fast path: if no target chars exist, return original.
        // (Still returns String to match your signature.)
        if s2t {
            if !sv.contains('“') && !sv.contains('”') && !sv.contains('‘') && !sv.contains('’')
            {
                return sv.to_owned();
            }
        } else {
            if !sv.contains('「') && !sv.contains('」') && !sv.contains('『') && !sv.contains('』')
            {
                return sv.to_owned();
            }
        }

        let mut out = String::with_capacity(sv.len());
        if s2t {
            for ch in sv.chars() {
                match ch {
                    '“' => out.push('「'),
                    '”' => out.push('」'),
                    '‘' => out.push('『'),
                    '’' => out.push('』'),
                    _ => out.push(ch),
                }
            }
        } else {
            for ch in sv.chars() {
                match ch {
                    '「' => out.push('“'),
                    '」' => out.push('”'),
                    '『' => out.push('‘'),
                    '』' => out.push('’'),
                    _ => out.push(ch),
                }
            }
        }

        out
    }

    #[inline]
    fn convert_punctuation_cow<'a>(sv: &'a str, config: &str) -> Cow<'a, str> {
        let s2t = config.starts_with('s') || !config.contains('s');

        let needs = if s2t {
            sv.contains('“') || sv.contains('”') || sv.contains('‘') || sv.contains('’')
        } else {
            sv.contains('「') || sv.contains('」') || sv.contains('『') || sv.contains('』')
        };

        if !needs {
            return Cow::Borrowed(sv);
        }

        let mut out = String::with_capacity(sv.len());
        if s2t {
            for ch in sv.chars() {
                out.push(match ch { '“' => '「', '”' => '」', '‘' => '『', '’' => '』', _ => ch });
            }
        } else {
            for ch in sv.chars() {
                out.push(match ch { '「' => '“', '」' => '”', '『' => '‘', '』' => '’', _ => ch });
            }
        }
        Cow::Owned(out)
    }

}

pub fn find_max_utf8_length(sv: &str, max_byte_count: usize) -> usize {
    // 1. No longer than max byte count
    if sv.len() <= max_byte_count {
        return sv.len();
    }
    // 2. Longer than byte count
    let mut byte_count = max_byte_count;
    while byte_count > 0 && (sv.as_bytes()[byte_count] & 0b11000000) == 0b10000000 {
        byte_count -= 1;
    }
    byte_count
}

pub fn format_thousand(n: usize) -> String {
    let mut result_str = n.to_string();
    let mut offset = result_str.len() % 3;
    if offset == 0 {
        offset = 3;
    }

    while offset < result_str.len() {
        result_str.insert(offset, ',');
        offset += 4; // Including the added comma
    }
    result_str
}
