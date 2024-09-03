use anyhow::{anyhow, Result};

use crate::values::Value::{self, *};

macro_rules! define_builtins {
    (
        $(
            $variant:ident
            $token:literal 
            {
                $(
                    $args_pat:pat => $body:tt
                ),+
                $(,)?
            }
        ),+ 
        $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum BuiltIn {
            $($variant),+
        }

        impl TryFrom<&str> for BuiltIn {
            type Error = anyhow::Error;

            fn try_from(s: &str) -> Result<Self> {
                match s {
                    $($token => Ok(BuiltIn::$variant),)+
                    _ => Err(anyhow!("Unknown builtin: {}", s)),
                }
            }
        }

        impl BuiltIn {
            #[allow(unused_braces)]
            pub fn call(&self, args: Vec<Value>) -> Result<Value> {
                match self {
                    $(BuiltIn::$variant => { // Each builtin by symbol, eg +
                        match args.as_slice() {
                            $(
                                $args_pat => { Ok($body) },
                            )+
                            _ => Err(anyhow!("Invalid arguments {args:?} for builtin: {}", stringify!($variant))),
                        }
                    },)+
                }
            }
        }
    };
}

define_builtins!{
    // Arithmetic
    Plus "+" {
       [Number(a), Number(b)] => { Number(a + b) },
       [String(a), String(b)] => { 
            let mut result = std::string::String::new();
            result.push_str(&a);
            result.push_str(&b);
            String(result)
       },
    },
    Minus "-" {
        [Number(a), Number(b)] => { Number(a - b) },
        [Number(v)] => { Number(-v) },
    },
    Times "*" {
        [Number(a), Number(b)] => { Number(a * b) },
    },
    Divide "/" {
        [Number(a), Number(b)] => { Number(a / b) },
    },

    // Boolean
    And "and" {
        [Bool(a), Bool(b)] => { Bool(*a && *b) },
    },
    Or "or" {
        [Bool(a), Bool(b)] => { Bool(*a || *b) },
    },
    Not "!" {
        [Bool(v)] => { Bool(!v) },
    },

    // Comparisons
    LessThan "<" {
        [Number(a), Number(b)] => { Bool(a < b) },
    },
    LessThanOrEqual "<=" {
        [Number(a), Number(b)] => { Bool(a <= b) },
    },
    GreaterThanOrEqual ">=" {
        [Number(a), Number(b)] => { Bool(a >= b) },
    },
    GreaterThan ">" {
        [Number(a), Number(b)] => { Bool(a > b) },
    },
    Equal "==" {
        [a, b] => { Bool(a == b) },
    },
    NotEqual "!=" {
        [a, b] => { Bool(a != b) },
    },
    
    // I/O
    Print "print" {
        [Number(n)] => { println!("{}", n); Nil },
        [a] => { println!("{}", a); Nil },
    },
}