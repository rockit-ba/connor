//! 自定义错误

use std::fmt::{Display, Formatter};

pub struct Byte2JsonErr;

impl Display for Byte2JsonErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Byte To Json Fail ！")
    }
}

pub struct Struct2JsonErr;

impl Display for Struct2JsonErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Struct To Json Fail ！")
    }
}

pub struct Json2StructErr;

impl Display for Json2StructErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Json To Struct Fail ！")
    }
}