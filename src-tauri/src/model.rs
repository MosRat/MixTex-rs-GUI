pub const ENCODER_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    r"/../models/encoder_model.onnx"
));
// const ENCODER_BYTES: &[u8] = &[1];
pub const DECODER_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    r"/../models/decoder_model_merged.onnx"
));
// const DECODER_BYTES: &[u8] = &[1];
pub const TOKENIZER_STR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    r"/../models/tokenizer/tokenizer.json"
));
