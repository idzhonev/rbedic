// Copyright 2018 Ivan Dzhonev
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::prelude::*;
use std::fs::File;
use regex::Regex;
use std::cmp::Ordering::{self, Equal, Greater, Less};
//use log;

const PREFIX_NUMBER: usize = 200;

#[derive(Clone, Debug, Eq)]
pub struct DictDB {
    pub word: String,
    pub translation: String,
}

impl DictDB {
    /// Load database files into memory
    pub fn new() -> Vec<DictDB> {
        // read from files
        let mut string_en_bg = String::new();
        let mut string_bg_en = String::new();
        // TODO: Add Windows support
        {
            info!("Loading dictionaries");
            let mut file_en_bg = File::open("/usr/local/share/bedic/en_bg-utf8.dat")
                .expect("Unable to open file en_bg-utf8.dat");
            let mut file_bg_en = File::open("/usr/local/share/bedic/bg_en-utf8.dat")
                .expect("Unable to open file bg_en-utf8.dat");

            file_en_bg
                .read_to_string(&mut string_en_bg)
                .expect("Unable to read file en_bg-utf8.dat");
            file_bg_en
                .read_to_string(&mut string_bg_en)
                .expect("Unable to read file bg_en-utf8.dat");
        }
        info!("Parse en_bg-utf8.dat");
        let mut vector_en_bg = parse(&string_en_bg);
        vector_en_bg.sort();
        info!("Parse bg_en-utf8.dat");
        let mut vector_bg_en = parse(&string_bg_en);
        vector_bg_en.sort();

        let concatenated_dictionaries = [&vector_en_bg[..], &vector_bg_en[..]].concat();
        info!(
            "This database contains {} elements",
            concatenated_dictionaries.len()
        );
        //concatenated_dictionaries.sort();
        info!("Done");
        concatenated_dictionaries
    }

    #[inline(always)]
    pub fn search(searched_txt: &str, data: &Vec<DictDB>) -> Result<Vec<DictDB>, Vec<DictDB>> {
        //let mut vec_dict_db: Vec<DictDB> = Vec::new();

        let search_struct = DictDB {
            word: searched_txt.to_string().to_uppercase(),
            translation: "_".to_string(),
        };
        match data.my_binary_search(&search_struct) {
            Ok(vec_result) => return Ok(vec_result),
            Err(vec_result_err) => return Err(vec_result_err),
        };
    }
}

impl Ord for DictDB {
    #[inline(always)]
    fn cmp(&self, other: &DictDB) -> Ordering {
        self.word.cmp(&other.word)
    }
}

impl PartialOrd for DictDB {
    fn partial_cmp(&self, other: &DictDB) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DictDB {
    fn eq(&self, other: &DictDB) -> bool {
        self.word == other.word
    }
}

// see https://github.com/rust-lang/rust/blob/master/src/libcore/slice/mod.rs
trait VecDictDB {
    type Item;

    fn my_binary_search(&self, x: &DictDB) -> Result<Self::Item, Self::Item>
    where
        Self::Item: Ord;

    fn my_binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a DictDB) -> Ordering;
}

impl VecDictDB for Vec<DictDB> {
    type Item = Vec<DictDB>;

    #[inline(always)]
    fn my_binary_search(&self, x: &DictDB) -> Result<Self::Item, Self::Item>
    where
        DictDB: Ord,
    {
        let vec_data = self;
        let size = vec_data.len();
        // Search for prefix x
        match self.my_binary_search_by(|p| p.cmp(x)) {
            Ok(index) => {
                let mut result_vector: Vec<DictDB> = Vec::new();
                let mut matched_suffix: bool = false;

                let mut j: usize = index; // Start
                let mut number = j + PREFIX_NUMBER; // End
                if number >= size {
                    number = size;
                };
                for i in j..number {
                    let s = vec_data[i].word.clone();
                    if s.as_str().starts_with(&x.word) {
                        matched_suffix = true;
                        result_vector.push(vec_data[i].clone());
                    } else {
                        if matched_suffix {
                            break;
                        }
                    }

                    j += 1;
                }
                Ok(result_vector)
            }
            Err(index_err) => {
                let mut result_vector: Vec<DictDB> = Vec::new();
                let mut matched_suffix: bool = false;

                let mut j: usize = index_err; // Start
                let mut number = j + PREFIX_NUMBER; // End
                if number >= size {
                    number = size;
                };
                for i in j..number {
                    let s = vec_data[i].word.clone();
                    if s.as_str().starts_with(&x.word) {
                        matched_suffix = true;
                        result_vector.push(vec_data[i].clone());
                    } else {
                        if matched_suffix {
                            break;
                        }
                    }

                    j += 1;
                }
                Err(result_vector)
            }
        }
    }

    #[inline(always)]
    fn my_binary_search_by<'a, F>(&'a self, mut f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a DictDB) -> Ordering,
    {
        let s = self;
        let mut size = s.len();
        if size == 0 {
            return Err(0);
        }
        let mut base = 0usize;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            // mid is always in [0, size).
            // mid >= 0: by definition
            // mid < size: mid = size / 2 + size / 4 + size / 8 ...
            let cmp = f(unsafe { s.get_unchecked(mid) });
            base = if cmp == Greater { base } else { mid };
            size -= half;
        }
        // base is always in [0, size) because base <= mid.
        let cmp = f(unsafe { s.get_unchecked(base) });
        if cmp == Equal {
            Ok(base)
        } else {
            Err(base + (cmp == Less) as usize)
        }
    }
}

/// Parse files
fn parse(string_data: &str) -> Vec<DictDB> {
    let mut vec_dict_db: Vec<DictDB> = Vec::new();
    let data: Vec<String> = string_data.split("^;").map(|s| s.to_string()).collect();
    for i in &data {
        lazy_static! {
            static ref ITEM: Regex = Regex::new(r"^(.*)\n(.|\s)*").unwrap();
        }
        if ITEM.is_match(i) {
            for cap in ITEM.captures(i) {
                let dict_db = DictDB {
                    word: cap[1].to_string(),
                    translation: cap[0].to_string(),
                };
                vec_dict_db.push(dict_db);
            }
        }
    }
    vec_dict_db
}
