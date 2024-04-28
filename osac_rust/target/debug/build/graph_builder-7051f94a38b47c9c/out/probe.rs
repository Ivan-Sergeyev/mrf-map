
#![feature(doc_cfg)]
#![allow(dead_code)]

#[doc(cfg(feature = "some_feature"))]
pub struct Foobar;
