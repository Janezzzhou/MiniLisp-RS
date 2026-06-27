use eframe::egui;
use eframe::egui::text::{LayoutJob, TextFormat};

use crate::theme::GuiTheme;

/// Extra editor decorations such as current-line and bracket highlights.
#[derive(Clone, Copy, Default)]
pub struct EditorDecorations {
    current_line: Option<(usize, usize)>,
    matched_brackets: Option<(usize, usize)>,
}

/// Build editor decorations from the current cursor position.
pub fn editor_decorations(source: &str, cursor_index: Option<usize>) -> EditorDecorations {
    let Some(cursor_index) = cursor_index else {
        return EditorDecorations::default();
    };

    EditorDecorations {
        current_line: line_char_range(source, cursor_index),
        matched_brackets: matched_bracket_pair(source, cursor_index),
    }
}

/// Convert Lisp source into a syntax-highlighted LayoutJob.
pub fn highlight_lisp(
    source: &str,
    theme: &GuiTheme,
    decorations: EditorDecorations,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch == ';' {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            append_colored(
                &mut job,
                &chars[start..i],
                start,
                theme.syntax_comment_color,
                theme,
                decorations,
            );
            continue;
        }

        if ch == '"' {
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 2;
                    continue;
                }
                let current = chars[i];
                i += 1;
                if current == '"' {
                    break;
                }
            }
            append_colored(
                &mut job,
                &chars[start..i],
                start,
                theme.syntax_string_color,
                theme,
                decorations,
            );
            continue;
        }

        if ch.is_whitespace() {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            append_colored(
                &mut job,
                &chars[start..i],
                start,
                theme.syntax_identifier_color,
                theme,
                decorations,
            );
            continue;
        }

        if is_punctuation(ch) {
            append_colored(
                &mut job,
                &chars[i..i + 1],
                i,
                theme.syntax_punctuation_color,
                theme,
                decorations,
            );
            i += 1;
            continue;
        }

        let start = i;
        i += 1;
        while i < chars.len() && !chars[i].is_whitespace() && !is_token_boundary(chars[i]) {
            i += 1;
        }

        let token: String = chars[start..i].iter().collect();
        let color = classify_token(&token, theme);
        append_colored(&mut job, &chars[start..i], start, color, theme, decorations);
    }

    job
}

fn classify_token(token: &str, theme: &GuiTheme) -> egui::Color32 {
    if token == "#t" || token == "#f" {
        return theme.syntax_boolean_color;
    }

    if token == "." || token.parse::<f64>().is_ok() {
        return theme.syntax_number_color;
    }

    if is_special_form(token) {
        return theme.syntax_special_form_color;
    }

    if is_builtin(token) {
        return theme.syntax_builtin_color;
    }

    theme.syntax_identifier_color
}

/// Split runs only when formatting actually changes.
fn append_colored(
    job: &mut LayoutJob,
    text: &[char],
    start_char_index: usize,
    color: egui::Color32,
    theme: &GuiTheme,
    decorations: EditorDecorations,
) {
    let mut segment_start = 0;
    let mut current_format = decorated_format(start_char_index, color, theme, decorations);

    for index in 1..=text.len() {
        let next_format = if index < text.len() {
            decorated_format(start_char_index + index, color, theme, decorations)
        } else {
            current_format.clone()
        };

        if index == text.len() || next_format != current_format {
            let snippet: String = text[segment_start..index].iter().collect();
            job.append(&snippet, 0.0, current_format.clone());
            segment_start = index;
            current_format = next_format;
        }
    }
}

fn decorated_format(
    char_index: usize,
    color: egui::Color32,
    theme: &GuiTheme,
    decorations: EditorDecorations,
) -> TextFormat {
    let mut format = TextFormat {
        font_id: egui::FontId::monospace(theme.editor_font_size),
        color,
        ..Default::default()
    };

    if decorations
        .current_line
        .is_some_and(|(start, end)| start <= char_index && char_index < end)
    {
        format.background = theme.current_line_highlight;
    }

    if decorations
        .matched_brackets
        .is_some_and(|(left, right)| char_index == left || char_index == right)
    {
        format.background = theme.matched_bracket_fill;
        format.underline = theme.matched_bracket_stroke;
    }

    format
}

fn line_char_range(source: &str, cursor_index: usize) -> Option<(usize, usize)> {
    let chars: Vec<char> = source.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let clamped = cursor_index.min(chars.len().saturating_sub(1));
    let mut start = clamped;
    let mut end = clamped;

    while start > 0 && chars[start - 1] != '\n' {
        start -= 1;
    }

    while end < chars.len() && chars[end] != '\n' {
        end += 1;
    }

    Some((start, end.max(start + 1)))
}

fn matched_bracket_pair(source: &str, cursor_index: usize) -> Option<(usize, usize)> {
    let chars: Vec<char> = source.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let ignored = ignored_positions(&chars);
    let candidate = find_bracket_candidate(&chars, &ignored, cursor_index)?;
    let ch = chars[candidate];

    match ch {
        '(' => scan_forward_for_match(&chars, &ignored, candidate, '(', ')')
            .map(|matched| (candidate, matched)),
        ')' => scan_backward_for_match(&chars, &ignored, candidate, '(', ')')
            .map(|matched| (matched, candidate)),
        _ => None,
    }
}

/// Ignore brackets that appear inside strings or comments.
fn ignored_positions(chars: &[char]) -> Vec<bool> {
    let mut ignored = vec![false; chars.len()];
    let mut in_string = false;
    let mut in_comment = false;
    let mut i = 0;

    while i < chars.len() {
        if in_comment {
            ignored[i] = true;
            if chars[i] == '\n' {
                in_comment = false;
            }
            i += 1;
            continue;
        }

        if in_string {
            ignored[i] = true;
            if chars[i] == '\\' && i + 1 < chars.len() {
                ignored[i + 1] = true;
                i += 2;
                continue;
            }
            if chars[i] == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if chars[i] == ';' {
            ignored[i] = true;
            in_comment = true;
            i += 1;
            continue;
        }

        if chars[i] == '"' {
            ignored[i] = true;
            in_string = true;
            i += 1;
            continue;
        }

        i += 1;
    }

    ignored
}

fn find_bracket_candidate(chars: &[char], ignored: &[bool], cursor_index: usize) -> Option<usize> {
    let candidates = [cursor_index, cursor_index.saturating_sub(1)];

    for &index in &candidates {
        if index < chars.len() && !ignored[index] && matches!(chars[index], '(' | ')') {
            return Some(index);
        }
    }

    None
}

fn scan_forward_for_match(
    chars: &[char],
    ignored: &[bool],
    start: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0;

    for i in start..chars.len() {
        if ignored[i] {
            continue;
        }
        if chars[i] == open {
            depth += 1;
        } else if chars[i] == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }

    None
}

fn scan_backward_for_match(
    chars: &[char],
    ignored: &[bool],
    start: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0;

    for i in (0..=start).rev() {
        if ignored[i] {
            continue;
        }
        if chars[i] == close {
            depth += 1;
        } else if chars[i] == open {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }

    None
}

fn is_punctuation(ch: char) -> bool {
    matches!(ch, '(' | ')' | '\'' | '`' | ',')
}

fn is_token_boundary(ch: char) -> bool {
    matches!(ch, '(' | ')' | '\'' | '`' | ',' | '"' | ';')
}

fn is_special_form(token: &str) -> bool {
    matches!(
        token,
        "begin"
            | "cond"
            | "define"
            | "lambda"
            | "let"
            | "quasiquote"
            | "quote"
            | "if"
            | "and"
            | "or"
            | "unquote"
    )
}

fn is_builtin(token: &str) -> bool {
    matches!(
        token,
        "append"
            | "abs"
            | "apply"
            | "atom?"
            | "boolean?"
            | "car"
            | "cdr"
            | "cons"
            | "display"
            | "displayln"
            | "eq?"
            | "equal?"
            | "error"
            | "even?"
            | "eval"
            | "expt"
            | "exit"
            | "filter"
            | "integer?"
            | "length"
            | "list"
            | "list?"
            | "map"
            | "newline"
            | "not"
            | "null?"
            | "number?"
            | "odd?"
            | "pair?"
            | "print"
            | "procedure?"
            | "quotient"
            | "reduce"
            | "remainder"
            | "string?"
            | "symbol?"
            | "zero?"
            | "+"
            | "-"
            | "*"
            | "/"
            | "="
            | "<"
            | "<="
            | "modulo"
            | ">"
            | ">="
    )
}
