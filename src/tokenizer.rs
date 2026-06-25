use crate::error::LispError;
use crate::token::Token;
use std::iter::Peekable;
use std::str::Chars;

const TOKEN_END: [char; 6] = ['(', ')', '\'', '`', ',', '"'];

//对外接口，用于将输入字符串转换为Token向量
pub fn tokenize(input: &str) -> Result<Vec<Token>, LispError> {
    Tokenizer::new(input).tokenize()
}

//Token解析器结构体
struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,     // 输入字符串的字符迭代器
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Tokenizer {
            chars: input.chars().peekable(),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, LispError> {  // 主循环，收集所有Token
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {    //当还有字符时
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token>, LispError> {  // 提取下一个Token
        while let Some(&ch) = self.chars.peek() { //当还有字符要处理时
            // ;后内容全忽略（跳过注释）
            if ch == ';' {
                self.chars.next();
                while let Some(c2) = self.chars.next() {
                    if c2 == '\n' {
                        break;
                    }
                }
                continue;
            }
            //跳过注释
            if ch.is_whitespace() {
                self.chars.next();
                continue;
            }
            // 单字符Token
            if let Some(token) = self.token_from_char(ch) {
                self.chars.next();
                return Ok(Some(token));
            }
            // 布尔字面量
            if ch == '#' {
                self.chars.next();
                let value: char  = match self.chars.next(){
                    Some(c) => Ok(c),
                    None => Err(LispError::SyntaxError("Unexpected character after #".into())),
                }?;
                return Ok(Some(match value {
                    't' => Token::BooleanLiteral(true),
                    'f' => Token::BooleanLiteral(false),
                    _ => return Err(LispError::SyntaxError("Unexpected character after #".into())),
                }));
            }
            // 字符串字面量
            if ch == '"' {
                self.chars.next();
                return Ok(Some(Token::StringLiteral(self.read_string()?)));
            }
            // 符号/标识符/数字
            return Ok(Some(self.read_symbol()?));
        }

        Ok(None)
    }

    // 从字符创建Token
    fn token_from_char(&self, ch: char) -> Option<Token> {
        match ch {
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            '\'' => Some(Token::Quote),
            '`' => Some(Token::Quasiquote),
            ',' => Some(Token::Unquote),
            _ => None,
        }
    }

    // 读取字符串字面量
    fn read_string(&mut self) -> Result<String, LispError> {
        let mut result = String::new();
        while let Some(ch) = self.chars.next() {
            match ch {
                '"' => return Ok(result),
                '\\' => {
                    let escaped = self
                        .chars
                        .next()
                        .ok_or_else(|| LispError::SyntaxError("Unexpected end of string literal".into()))?;
                    match escaped {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        other => result.push(other),
                    }
                }
                other => result.push(other),
            }
        }
        Err(LispError::SyntaxError("Unexpected end of string literal".into()))
    }

    // 读取符号/标识符/数字
    fn read_symbol(&mut self) -> Result<Token, LispError> {
        let mut text = String::new();
        while let Some(&next) = self.chars.peek() {
            if next.is_whitespace() || TOKEN_END.contains(&next) {
                break;
            }
            text.push(next);
            self.chars.next();
        }

        if text == "." {
            return Ok(Token::Dot);
        }

        if text.is_empty() {
            return Err(LispError::SyntaxError("Unexpected empty token".into()));
        }

        if text.starts_with(|c: char| c.is_ascii_digit() || c == '+' || c == '-' || c == '.') {
            if let Ok(value) = text.parse::<f64>() {
                return Ok(Token::NumericLiteral(value));
            }
        }
        // 解析完成
        Ok(Token::Identifier(text))
    }
}
