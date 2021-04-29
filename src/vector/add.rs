use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::mem::MaybeUninit;
use std::ops::Add;
use std::ops::Index;
use std::ops::Sub;

use crate::array::raw_arr_zipwith;
use crate::vector::Vector;
