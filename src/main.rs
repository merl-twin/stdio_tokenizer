#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate tokenizer;
extern crate rust_stemmers;
//extern crate snap;
//extern crate time;

use std::io::{BufReader,BufRead};

use tokenizer::{Token,Number,Numerical,IntoTokenizer};
use rust_stemmers::{Algorithm, Stemmer};

use std::collections::HashMap;

pub enum Language {
    Russian,
}
    
struct StemmerEx {
    stemmer:     Stemmer,
    exceptions:  Option<HashMap<String,String>>,
}
impl StemmerEx {
    fn new(language: Language) -> StemmerEx {
        match language {
            Language::Russian =>  {
                let mut d=HashMap::new();
                d.insert("газпром".to_string(),"газпром".to_string());
                d.insert("ростелеком".to_string(),"ростелеком".to_string());
                d.insert("кредит".to_string(),"кредит".to_string());
                StemmerEx{
                    stemmer:     Stemmer::create(Algorithm::Russian),
                    exceptions:  Some(d),
                }
            },
        }
    }
    fn stem(&self, word: &str) -> String {
        if let Some(ref dict) = self.exceptions {
            if let Some(val) = dict.get(word) {
                return val.clone();
            }
        }
        self.stemmer.stem(word).to_string()
    }
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Representation {
    Word {
        word: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        stem: Option<String>
    },
    Numerical { word: String, subtype: String },
    Number { word: String },
    StrangeWord { word: String },
    Emoji { word: String },
    Unicode { word: String },
    Hashtag { word: String },
    Mention { word: String },
    Url { word: String },
    BBCode {
        text: Vec<Representation>,
        data: Vec<Representation>,
    },
}

fn text_to_words(text: &str) -> Vec<Representation> {
    fn tok2tok(tok: Token, stemmer: &StemmerEx) -> Option<Representation> {
        match tok {
            Token::Separator(..) | Token::Punctuation(..) => None,
            Token::Word(w) => Some(Representation::Word { stem: {
                let stem = stemmer.stem(&w.to_lowercase());
                if stem!=w { Some(stem) } else { None }
            },  word: w }),
            Token::StrangeWord(w) => Some(Representation::StrangeWord { word: w }),
            Token::Numerical(Numerical::DotSeparated(w)) => Some(Representation::Numerical { word: format!("{}",w), subtype: "dotseparated".to_string() }),
            Token::Numerical(Numerical::Measures(w)) => Some(Representation::Numerical { word: format!("{}",w), subtype: "measures".to_string() }),
            Token::Numerical(Numerical::Alphanumeric(w)) => Some(Representation::Numerical { word: format!("{}",w), subtype: "alphanumeric".to_string() }),
            Token::Number(Number::Integer(n)) => Some(Representation::Number { word: format!("{}",n) }),
            Token::Number(Number::Float(n)) => Some(Representation::Number { word: format!("{}",n) }),
            Token::Emoji(w) => Some(Representation::Emoji { word: w }),
            Token::Hashtag(w) => Some(Representation::Hashtag { word: w }),
            Token::Mention(w) => Some(Representation::Mention { word: w }),
            Token::Unicode(w) => Some(Representation::Unicode { word: w }),
            Token::Url(w) => Some(Representation::Url { word: w }),
            Token::BBCode { text, data } => Some(Representation::BBCode {
                text: text.into_iter().filter_map(|tok| tok2tok(tok,&stemmer)).collect(),
                data: data.into_iter().filter_map(|tok| tok2tok(tok,&stemmer)).collect(),
            }),
        }
    }
    
    let stemmer = StemmerEx::new(Language::Russian);
    match text
        .to_lowercase()
        .into_tokens() {
            Ok(tokens) => tokens
                .filter_map(|tok| tok2tok(tok.token,&stemmer))
                .collect(),
            Err(_) => Vec::new(),
        }
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(untagged)]
pub enum IOid {
    Unsigned(u64),
    String(String),
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Input {
    id: IOid,
    text: String,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Output {
    id: IOid,
    words: Vec<Representation>,
}

#[derive(Debug)]
pub enum Error {
    Read(std::io::Error),
    Json(serde_json::Error),
}


fn main() -> Result<(),Error> {
    for row in BufReader::new(std::io::stdin()).lines() {
        let inp: Input = serde_json::from_slice(row.map_err(Error::Read)?.as_bytes()).map_err(Error::Json)?;
        let out = Output {
            id: inp.id,
            words: text_to_words(&inp.text),
        };
        println!("{}",serde_json::to_string(&out).map_err(Error::Json)?);
    }
    Ok(())
}
