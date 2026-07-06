use unicode_width::UnicodeWidthStr;

pub fn render_markdown_plain(markdown: &str) -> String {
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            out.push(format!("  {line}"));
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            out.push(trimmed.trim_start_matches('#').trim_start().to_string());
        } else {
            out.push(line.replace('`', ""));
        }
    }
    out.join("\n").trim_end().to_string()
}

pub fn byte_index(value: &str, char_index: usize) -> usize {
    value
        .char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(value.len())
}

pub fn char_len(value: &str) -> usize {
    value.chars().count()
}

pub fn prefix(value: &str, char_index: usize) -> String {
    value.chars().take(char_index).collect()
}

pub fn display_width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

const CHO: &[char] = &[
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
];
const JUNG: &[char] = &[
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ',
    'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];
const JONG: &[char] = &[
    '\0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ',
    'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

pub fn compose_hangul_jamo(value: &str) -> String {
    let mut out = String::new();
    let mut run = Vec::new();
    for char in decompose_hangul(value).chars() {
        if is_hangul_jamo(char) {
            run.push(char);
        } else {
            if !run.is_empty() {
                out.push_str(&compose_hangul_run(&run));
                run.clear();
            }
            out.push(char);
        }
    }
    if !run.is_empty() {
        out.push_str(&compose_hangul_run(&run));
    }
    out
}

fn decompose_hangul(value: &str) -> String {
    let mut chars = String::new();
    for char in value.chars() {
        let code = char as u32;
        if (0xAC00..=0xD7A3).contains(&code) {
            let offset = code - 0xAC00;
            let lead = (offset / 588) as usize;
            let vowel = ((offset % 588) / 28) as usize;
            let tail = (offset % 28) as usize;
            chars.push(CHO[lead]);
            chars.push(JUNG[vowel]);
            if tail != 0 {
                chars.push(JONG[tail]);
            }
        } else {
            chars.push(char);
        }
    }
    chars
}

fn compose_hangul_run(chars: &[char]) -> String {
    let mut out = String::new();
    let mut lead = None;
    let mut vowel = None;
    let mut tail = None;

    fn emit(
        out: &mut String,
        lead: &mut Option<char>,
        vowel: &mut Option<char>,
        tail: &mut Option<char>,
    ) {
        match (*lead, *vowel) {
            (Some(l), Some(v)) => {
                if let (Some(l_index), Some(v_index)) = (cho_index(l), jung_index(v)) {
                    let code = 0xAC00
                        + ((l_index * 21 + v_index) * 28 + tail.and_then(jong_index).unwrap_or(0))
                            as u32;
                    if let Some(char) = char::from_u32(code) {
                        out.push(char);
                    }
                } else {
                    for part in [*lead, *vowel, *tail].into_iter().flatten() {
                        out.push(part);
                    }
                }
            }
            _ => {
                for part in [*lead, *vowel, *tail].into_iter().flatten() {
                    out.push(part);
                }
            }
        }
        *lead = None;
        *vowel = None;
        *tail = None;
    }

    for &char in chars {
        if jung_index(char).is_some() {
            if lead.is_none() {
                out.push(char);
            } else if vowel.is_none() {
                vowel = Some(char);
            } else if tail.is_none() {
                if let Some(combined) = combine_jung(vowel.unwrap(), char) {
                    vowel = Some(combined);
                } else {
                    emit(&mut out, &mut lead, &mut vowel, &mut tail);
                    out.push(char);
                }
            } else if let Some((first, second)) = split_jong(tail.unwrap()) {
                tail = Some(first);
                emit(&mut out, &mut lead, &mut vowel, &mut tail);
                lead = Some(second);
                vowel = Some(char);
            } else {
                let next_lead = tail;
                tail = None;
                emit(&mut out, &mut lead, &mut vowel, &mut tail);
                lead = next_lead;
                vowel = Some(char);
            }
        } else if lead.is_none() {
            lead = Some(char);
        } else if vowel.is_none() {
            if let Some(combined) = combine_cho(lead.unwrap(), char) {
                lead = Some(combined);
            } else {
                emit(&mut out, &mut lead, &mut vowel, &mut tail);
                lead = Some(char);
            }
        } else if tail.is_none() && jong_index(char).is_some() {
            tail = Some(char);
        } else if tail.is_some() {
            if let Some(combined) = combine_jong(tail.unwrap(), char) {
                tail = Some(combined);
            } else {
                emit(&mut out, &mut lead, &mut vowel, &mut tail);
                lead = Some(char);
            }
        } else {
            emit(&mut out, &mut lead, &mut vowel, &mut tail);
            lead = Some(char);
        }
    }
    emit(&mut out, &mut lead, &mut vowel, &mut tail);
    out
}

fn is_hangul_jamo(char: char) -> bool {
    cho_index(char).is_some() || jung_index(char).is_some() || jong_index(char).is_some()
}

fn cho_index(char: char) -> Option<usize> {
    CHO.iter().position(|value| *value == char)
}

fn jung_index(char: char) -> Option<usize> {
    JUNG.iter().position(|value| *value == char)
}

fn jong_index(char: char) -> Option<usize> {
    JONG.iter()
        .position(|value| *value == char)
        .filter(|index| *index != 0)
}

fn combine_cho(a: char, b: char) -> Option<char> {
    match (a, b) {
        ('ㄱ', 'ㄱ') => Some('ㄲ'),
        ('ㄷ', 'ㄷ') => Some('ㄸ'),
        ('ㅂ', 'ㅂ') => Some('ㅃ'),
        ('ㅅ', 'ㅅ') => Some('ㅆ'),
        ('ㅈ', 'ㅈ') => Some('ㅉ'),
        _ => None,
    }
}

fn combine_jung(a: char, b: char) -> Option<char> {
    match (a, b) {
        ('ㅗ', 'ㅏ') => Some('ㅘ'),
        ('ㅗ', 'ㅐ') => Some('ㅙ'),
        ('ㅗ', 'ㅣ') => Some('ㅚ'),
        ('ㅜ', 'ㅓ') => Some('ㅝ'),
        ('ㅜ', 'ㅔ') => Some('ㅞ'),
        ('ㅜ', 'ㅣ') => Some('ㅟ'),
        ('ㅡ', 'ㅣ') => Some('ㅢ'),
        _ => None,
    }
}

fn combine_jong(a: char, b: char) -> Option<char> {
    match (a, b) {
        ('ㄱ', 'ㅅ') => Some('ㄳ'),
        ('ㄴ', 'ㅈ') => Some('ㄵ'),
        ('ㄴ', 'ㅎ') => Some('ㄶ'),
        ('ㄹ', 'ㄱ') => Some('ㄺ'),
        ('ㄹ', 'ㅁ') => Some('ㄻ'),
        ('ㄹ', 'ㅂ') => Some('ㄼ'),
        ('ㄹ', 'ㅅ') => Some('ㄽ'),
        ('ㄹ', 'ㅌ') => Some('ㄾ'),
        ('ㄹ', 'ㅍ') => Some('ㄿ'),
        ('ㄹ', 'ㅎ') => Some('ㅀ'),
        ('ㅂ', 'ㅅ') => Some('ㅄ'),
        _ => None,
    }
}

fn split_jong(char: char) -> Option<(char, char)> {
    match char {
        'ㄳ' => Some(('ㄱ', 'ㅅ')),
        'ㄵ' => Some(('ㄴ', 'ㅈ')),
        'ㄶ' => Some(('ㄴ', 'ㅎ')),
        'ㄺ' => Some(('ㄹ', 'ㄱ')),
        'ㄻ' => Some(('ㄹ', 'ㅁ')),
        'ㄼ' => Some(('ㄹ', 'ㅂ')),
        'ㄽ' => Some(('ㄹ', 'ㅅ')),
        'ㄾ' => Some(('ㄹ', 'ㅌ')),
        'ㄿ' => Some(('ㄹ', 'ㅍ')),
        'ㅀ' => Some(('ㄹ', 'ㅎ')),
        'ㅄ' => Some(('ㅂ', 'ㅅ')),
        _ => None,
    }
}
