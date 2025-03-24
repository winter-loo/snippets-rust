fn main() {
    let mut f = Forth::new();
    assert!(f.eval(": foo 5 ;").is_ok());
    assert!(f.eval(": bar foo ;").is_ok());
    assert!(f.eval(": foo 6 ;").is_ok());
    assert!(f.eval("bar foo").is_ok());
    assert_eq!(f.stack(), [5, 6]);

    let mut f = Forth::new();
    assert!(f.eval(": foo 10 ;").is_ok());
    assert!(f.eval(": foo foo 1 + ;").is_ok());
    assert!(f.eval(": foo foo 1 + ;").is_ok());
    assert!(f.eval("foo").is_ok());

    let mut f = Forth::new();
    f.eval(": a 0 drop ;").unwrap();
    f.eval(": b a a ;").unwrap();
    f.eval(": c b b ;").unwrap();
    f.eval(": d c c ;").unwrap();
    f.eval(": e d d ;").unwrap();

    assert!(f.stack().is_empty());
}

use std::collections::{HashMap, VecDeque};

pub type Value = i32;
pub type Result = std::result::Result<(), Error>;

pub struct Forth {
    defs: HashMap<String, String>,
    stack: Vec<Value>,
}

struct Def {
    name: String,
    namespace: HashMap<String, u32>,
    value: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord,
    InvalidWord,
}

impl Forth {
    pub fn new() -> Forth {
        Forth {
            defs: HashMap::new(),
            stack: vec![],
        }
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    fn get_number(&mut self) -> std::result::Result<Value, Error> {
        match self.stack.pop() {
            None => Err(Error::StackUnderflow),
            Some(x) => Ok(x),
        }
    }

    fn rewrite_def(&self, def: &Vec<String>) -> String {
        let mut newdef = Vec::with_capacity(def.len());
        for token in def {
            if let Some(odef) = self.defs.get(token) {
                newdef.push(odef.to_string());
            } else {
                newdef.push(token.to_string());
            }
        }
        newdef.join(" ")
    }

    // 1 2 +
    pub fn eval(&mut self, input: &str) -> Result {
        let mut expr: VecDeque<String> = input.split(' ').map(|s| s.to_ascii_lowercase()).collect();
        // : FOO 1 2 3 ;
        if let Some(first) = expr.front() {
            if first == ":" {
                if expr.len() < 4 {
                    return Err(Error::InvalidWord);
                }
                if let Some(last) = expr.back() {
                    if last != ";" {
                        return Err(Error::InvalidWord);
                    }
                    expr.pop_back();
                    expr.pop_front();
                    let identifier = expr[0].to_string().to_ascii_lowercase();
                    if identifier.parse::<Value>().is_ok() {
                        return Err(Error::InvalidWord);
                    }
                    expr.pop_front();
                    let def = expr.into_iter().collect::<Vec<_>>();
                    let def = self.rewrite_def(&def);
                    self.defs.insert(identifier, def);
                    return Ok(());
                } else {
                    return Err(Error::InvalidWord);
                }
            }
        }
        while let Some(token) = expr.pop_front() {
            if let Ok(n) = token.parse::<Value>() {
                self.stack.push(n);
                continue;
            }
            if let Some(x) = self.defs.get(&token) {
                expr.extend(x.split(' ').map(|s| s.to_string()));
            } else {
                match &token[..] {
                    "+" => {
                        let op1 = self.get_number()?;
                        let op2 = self.get_number()?;
                        self.stack.push(op2 + op1);
                    }
                    "-" => {
                        let op1 = self.get_number()?;
                        let op2 = self.get_number()?;
                        self.stack.push(op2 - op1);
                    }
                    "*" => {
                        let op1 = self.get_number()?;
                        let op2 = self.get_number()?;
                        self.stack.push(op2 * op1);
                    }
                    "/" => {
                        let op1 = self.get_number()?;
                        let op2 = self.get_number()?;
                        if op1 == 0 {
                            return Err(Error::DivisionByZero);
                        }
                        self.stack.push(op2 / op1);
                    }
                    "dup" => {
                        let last = match self.stack.last() {
                            None => return Err(Error::StackUnderflow),
                            Some(x) => *x,
                        };
                        self.stack.push(last);
                    }
                    "drop" => {
                        if self.stack.pop().is_none() {
                            return Err(Error::StackUnderflow);
                        }
                    }
                    "swap" => {
                        let op1 = self.get_number()?;
                        let op2 = self.get_number()?;
                        self.stack.push(op1);
                        self.stack.push(op2);
                    }
                    "over" => {
                        let op1 = match self.stack.iter().nth_back(1) {
                            None => return Err(Error::StackUnderflow),
                            Some(x) => *x,
                        };

                        self.stack.push(op1);
                    }
                    _ => return Err(Error::UnknownWord),
                }
            }
        }
        Ok(())
    }
}
