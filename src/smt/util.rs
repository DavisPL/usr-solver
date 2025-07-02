/*
    Utilities for string parsing
*/

use regex::Regex;

use super::parse::SmtParseError;

pub fn hex_to_char(number: u64) -> Result<char, SmtParseError> {
    char::from_u32(number as u32).ok_or(SmtParseError::FileError(format!(
        "Invalid hex value: {}",
        number
    )))
}

pub fn parse_bad_newlines(text: &str)-> Result<String, SmtParseError>{
    // Regex pattern for newlines in set-info
    let newlines_re=Regex::new(r"(?s)\(set-info\ :source\ \|(.*)\|\)").unwrap();
    
    replace_all(&newlines_re, text, |caps: &regex::Captures| {
        //let innertext=&caps[1];
        //println!("start");
        //println!("{}",innertext);
        //println!("end");
        //let modified_innertext=innertext.replace("\n", " ").replace("\r", " ");
        Ok("".to_string())
        //Ok(format!("(set-info :source |{}|)",modified_innertext))
    })
    
}

pub fn parse_unicode_escape(text: &str) -> Result<String, SmtParseError> {
    // Regex pattern for unicode escapes \u{Hex}
    // Does not check invalid hex
    let unicode_escape_re = Regex::new(r"\\u\{([0-9A-Fa-f]+)\}").unwrap();

    replace_all(&unicode_escape_re, text, |caps: &regex::Captures| {
        // Unwrap is okay since regex check between 0-f for hex
        let hex_value = u32::from_str_radix(&caps[1], 16).unwrap();
        match char::from_u32(hex_value) {
            Some(v) => Ok(v.to_string()),
            // Error on invalid hex
            None => Err(SmtParseError::FileError(format!(
                "Bad hex in unicode escape {:?}",
                hex_value
            ))),
        }
    })
}

fn replace_all<E>(
    re: &Regex,
    haystack: &str,
    replacement: impl Fn(&regex::Captures) -> Result<String, E>,
) -> Result<String, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
        let m = caps.get(0).unwrap();
        new.push_str(&haystack[last_match..m.start()]);
        new.push_str(&replacement(&caps)?);
        last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(new)
}
