// faster than allocating Vec for partitions
#![feature(iter_partition_in_place)]
pub mod app;
pub mod arena;
pub mod components;
pub mod draw;
pub mod meta;
pub mod parse;
pub mod quadtree;
pub mod universe;
