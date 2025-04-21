use std::collections::HashMap;

use enigo::Key;

#[derive(Clone)]
pub enum Token {
    ShortPause,
    Pause,
    LongPause,
    Single(Key),
    Many(Vec<Key>),
    ManyFast(Vec<Key>),
}

#[derive(Debug)]
pub struct TokenDurations {
    pub short_pause: f64,
    pub pause: f64,
    pub long_pause: f64,
    pub single: f64,
    pub many_fast: f64,
}

#[derive(Clone)]
pub struct Header {
    pub title: Option<String>,
    pub writer: Option<String>,
    pub length: f64,
}

#[derive(Clone)]
pub struct Sheet {
    pub header: Header,
    pub tokens: Vec<Token>,
}

pub struct PauseDistribution {
    pub short: f64,
    pub standard: f64,
    pub long: f64,
    pub pause_ratio: f64,
    pub many_fast_proportion: f64,
}

pub fn calculate_token_durations(
    multiplier: f64,
    pause_distribution: &PauseDistribution,
) -> Result<TokenDurations, String> {
    if pause_distribution.pause_ratio <= 0.0 {
        return Err("Note-pause ratio must be greater than zero.".to_string());
    }

    let total_pause_distribution =
        pause_distribution.short + pause_distribution.standard + pause_distribution.long;
    if total_pause_distribution != 1.0 {
        return Err("Pause distribution percentages must add up to 1.0".to_string());
    }
    if pause_distribution.many_fast_proportion < 0.0
        || pause_distribution.many_fast_proportion > 1.0
    {
        return Err("many_fast_proportion must be between 0.0 and 1.0".to_string());
    }

    let note_proportion = pause_distribution.pause_ratio / (pause_distribution.pause_ratio + 1.0);
    let pause_proportion = 1.0 - note_proportion;

    let remaining_proportion = 1.0 - pause_distribution.many_fast_proportion;
    let single = note_proportion * remaining_proportion * multiplier;
    let pause_time = pause_proportion * remaining_proportion;

    let many_fast = pause_distribution.many_fast_proportion * multiplier;

    Ok(TokenDurations {
        short_pause: pause_time * pause_distribution.short,
        pause: pause_time * pause_distribution.standard,
        long_pause: pause_time * pause_distribution.long,

        single,
        many_fast,
    })
}

fn parse_tokens(output: &mut Vec<Token>, input: &str) -> Result<(), String> {
    let chars = input.chars();

    let mut in_many = false;
    let mut in_many_fast = false;
    let mut group: Option<Vec<Key>> = None;
    for character in chars {
        match character {
            '[' => {
                in_many = true;
                group = Some(Vec::new());
            }
            ']' => {
                if let Some(keys) = group.take() {
                    if in_many_fast {
                        output.push(Token::ManyFast(keys));
                    } else if in_many {
                        output.push(Token::Many(keys));
                    } else {
                        return Err("Attempted to close while not open".to_string());
                    }
                } else {
                    return Err("Attempted to close invalid key block".to_string());
                }
                in_many = false;
                in_many_fast = false;
            }
            '|' => output.push(Token::Pause),
            ' ' => {
                if in_many {
                    in_many_fast = true;
                } else {
                    output.push(Token::ShortPause);
                }
            }
            _ => {
                if let Some(keys) = &mut group {
                    keys.push(Key::Unicode(character));
                } else {
                    output.push(Token::Single(Key::Unicode(character)));
                }
            }
        }
    }

    Ok(())
}

pub fn parse_sheet(input: &str) -> Result<Sheet, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut defines: HashMap<&str, &str> = HashMap::new();

    let lines = input.lines();

    let mut last_line_empty = false;
    for line in lines {
        if line.is_empty() {
            if !last_line_empty {
                tokens.push(Token::LongPause);
            }
            last_line_empty = true;
            continue;
        } else {
            last_line_empty = false;
        }

        if line.chars().next().unwrap() == '#' {
            match line.split_once(' ') {
                None => return Err("Defines must be a name and value pair".to_string()),
                Some((k, v)) => defines.insert(k, v),
            };
            continue;
        }

        if let Err(x) = parse_tokens(&mut tokens, line) {
            return Err(x);
        }
    }

    let length = match defines.get("#length") {
        None => return Err("Sheet length must be defined".to_string()),
        Some(&length) => match length.split_once(':') {
            None => return Err("Invalid sheet length format".to_string()),
            Some((mins, secs)) => {
                let mins = match mins.parse::<f64>() {
                    Ok(x) => x,
                    Err(_) => return Err("Invalid sheet length minutes".to_string()),
                };
                let secs = match secs.parse::<f64>() {
                    Ok(x) => x,
                    Err(_) => return Err("Invalid sheet length seconds".to_string()),
                };
                mins * 60.0 + secs
            }
        },
    };

    let header = Header {
        title: match defines.get("#title") {
            None => None,
            Some(&x) => Some(x.to_string()),
        },
        writer: match defines.get("#writer") {
            None => None,
            Some(&x) => Some(x.to_string()),
        },
        length,
    };

    Ok(Sheet { tokens, header })
}
