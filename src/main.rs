use std::{iter::Peekable, fmt::Display, str::Chars};

// 自定义Result类型
pub type Result<T> = std::result::Result<T, ExpError>;

// 自定义错误类型
#[derive(Debug)]
pub enum ExpError {
    Parse(String),
}

impl std::error::Error for ExpError {}

impl Display for ExpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(s) => write!(f,"{}", s),
        }
    }
}



#[derive(Debug,Copy,Clone)]
enum Token {
    Number(i32),
    Plus,       // 加
    Minus,      // 减
    Multiply,   // 乘
    Divide,     // 除
    Power,      // 幂
    LeftParen,  // 左括号
    RightParen, // 右括号
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}" , 
            match self {
                Token::Number(n) => n.to_string(),
                Token::Plus => "+".to_string(),
                Token::Minus => "-".to_string(),
                Token::Multiply => "*".to_string(),
                Token::Divide => "/".to_string(),
                Token::Power => "^".to_string(),
                Token::LeftParen => "(".to_string(),
                Token::RightParen => ")".to_string(),
            }
        )
    }
}

const ASSOC_LEFT: i32 = 0;
const ASSOC_RIGHT: i32 = 1;

impl Token {
    // 判断是否是运算符
    fn is_oprator(&self) -> bool {
        match self {
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Power => true,
            _ => false
        }
    }

    // 获取运算符的优先级
    fn precedence(&self) -> i32 {
        match self {
            Token::Plus | Token::Minus => 1,
            Token::Multiply | Token::Divide => 2,
            Token::Power => 3,
            _ => 0,
        }
    }

    // 获取运算符的结合性
    fn assoc(&self) -> i32 {
        match self {
            Token::Power => ASSOC_RIGHT,
            _ => ASSOC_LEFT,
        }
    }

    // 根据运算符进行计算
    fn compute(&self, l: i32, r: i32) -> Option<i32> {
        match self {
            Token::Plus => Some(l + r),
            Token::Minus => Some(l - r),
            Token::Multiply => Some(l * r),
            Token::Divide => Some(l / r),
            Token::Power => Some(l.pow(r as u32)),
            _ => None,
        }
    } 
}

struct Tokenizer<'a> {
    tokens: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    fn new(expr: &'a str) -> Self {
        Self {
            tokens: expr.chars().peekable()
        }
    }

    // 消除空白字符串
    fn consume_whitespace(&mut self) {
        while let Some(&c) = self.tokens.peek() {
            if c.is_whitespace() {
                self.tokens.next();
            } else {
                break;
            }
        }
    }

    // 扫描数字
    fn scan_number(&mut self) -> Option<Token> {
        let mut num = String::new();

        while let Some(&c) = self.tokens.peek() {
            if c.is_numeric() {
                num.push(c);
                self.tokens.next();
            } else {
                break;
            }
        }

        match num.parse() {
            Ok(n) => Some(Token::Number(n)),
            Err(_) => None,
        }
    }

    // 扫描运算符号
    fn scan_operator(&mut self) -> Option<Token> {
        match self.tokens.next() {
            Some('+') => Some(Token::Plus),
            Some('-') => Some(Token::Minus),
            Some('*') => Some(Token::Multiply),
            Some('/') => Some(Token::Divide),
            Some('^') => Some(Token::Power),
            Some('(') => Some(Token::LeftParen),
            Some(')') => Some(Token::RightParen),
            _ => None,
        }
    }
}


impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        
        // 消除空白字符串
        self.consume_whitespace();
        // 解析当前token类型
        match self.tokens.peek() {
            Some(c) if c.is_numeric() => self.scan_number(),
            Some(_) => self.scan_operator(),
            None => None,
        }

    }
}

struct Expr<'a> {
    iter: Peekable<Tokenizer<'a>>
}

impl<'a> Expr<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            iter: Tokenizer::new(src).peekable(),
        }
    }

    // 计算单个Token或者子表达式
    fn compute_atom(&mut self) -> Result<i32> {
        match self.iter.peek() {
            Some(Token::Number(n)) => {
                let val = *n;
                self.iter.next();
                return Ok(val);
            },
            // 匹配左括号，递归计算括号内的表达式
            Some(Token::LeftParen) => {
                self.iter.next();
                let result = self.compute_expr(1)?;
                match self.iter.next() {
                    Some(Token::RightParen) => (),
                    _ => {
                        return Err(ExpError::Parse("Unexpected character".into()));
                    }
                }
                return Ok(result);
            }
            _ => {
                return Err(ExpError::Parse("123".to_string()));
            }
        }
    }

    fn compute_expr(&mut self,min_prec: i32) -> Result<i32> {
        // 计算第一个token
        let mut atom_lhs = self.compute_atom()?;

        loop {
            let cur_token = self.iter.peek();

            if cur_token.is_none() {
                break;
            }

            let token = *cur_token.unwrap();

            // token 此时一定是运算符
            // token的优先级大于等于min_prec
            if !token.is_oprator() || token.precedence() < min_prec {
                break;
            }

            let mut next_prec = token.precedence();

            if token.assoc() == ASSOC_LEFT {
                next_prec += 1;
            }

            self.iter.next();

            // 递归计算右边表达式的值
            let atom_rhs = self.compute_expr(next_prec)?;

            // 得到两边表达式的值，进行计算
            match token.compute(atom_lhs, atom_rhs) {
                Some(res) => atom_lhs = res,
                None => return Err(ExpError::Parse("Unexpected expr".into())),
            }

        }

        Ok(atom_lhs)
    }

    fn eval(&mut self) -> Result<i32> {
        let result = self.compute_expr(1)?;

        // 检查是否还有Token未进行处理，说明表达式错误
        if self.iter.peek().is_some() {
            return Err(ExpError::Parse("Unexpected end of expr".into()));
        }

        Ok(result)
    }

}

fn main() {
    let src = "10 * 2 ^ 2 + 5 + 20 / 4 + (1 + 1)";
    let mut expr = Expr::new(src);
    let result = expr.eval();
    println!("{:?}",result);
}