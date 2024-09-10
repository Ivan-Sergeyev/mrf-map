#![allow(dead_code)]

use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

pub trait Factor: Display + Index<usize> + IndexMut<usize> {
    fn arity(&self) -> usize;
    fn function_table_len(&self) -> usize;

    fn map(&self, mapping: fn(f64) -> f64) -> Self;
    fn map_inplace(&mut self, mapping: fn(&mut f64));

    fn new_zero_message(&self) -> Self;
    fn clone_for_message_passing(&self) -> Self;

    fn add_assign(&mut self, rhs: &Self);
    fn sub_assign(&mut self, rhs: &Self);
    fn mul_assign(&mut self, rhs: f64);

    fn add_assign_number(&mut self, rhs: f64);

    fn min(&self) -> f64;
}
