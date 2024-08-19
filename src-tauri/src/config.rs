use  ndarray::prelude::*;

pub struct Config{
    pub max_length:usize,
    pub num_layers:usize,
    pub hidden_size:usize,
    pub num_attention_head:usize,
    pub batch_size:usize,
    // pub decoder_start_input:Array<i64,Ix2>,
    pub stop_idx:usize,
}

pub const CONFIG:Config = Config{
    max_length:512,
    num_layers:3,
    hidden_size:768,
    num_attention_head:12,
    batch_size:1,
    // decoder_start_input:array![[0,0,30000_i64]],
    stop_idx:30000
};