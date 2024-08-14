use std::str::FromStr;

use ndarray::{prelude::*};
use ort::{GraphOptimizationLevel, Session, SessionOutputs};
use tokenizers::Tokenizer;

use crate::{check_repeat, DECODER_BYTES, ENCODER_BYTES, TOKENIZER_STR};

const MAX_LENGTH: usize = 512;
pub struct MixTexOnnx {
    encoder_session: Session,
    decoder_session: Session,
    tokenizer: Tokenizer,
}

impl MixTexOnnx {
    pub fn build() -> Result<Self, Box<dyn std::error::Error>> {
        let encoder_builder = Session::builder()?;
        let decoder_builder = Session::builder()?;

        // let _encoder_cuda = CUDAExecutionProvider::default()
        //     .with_device_id(0)
        //     .with_arena_extend_strategy(ort::ArenaExtendStrategy::NextPowerOfTwo)
        //     .with_memory_limit(2 * 1024 * 1024 * 1024)
        //     .with_conv_algorithm_search(ort::CUDAExecutionProviderCuDNNConvAlgoSearch::Exhaustive)
        //     .with_copy_in_default_stream(true);

        // let _decoder_cuda = CUDAExecutionProvider::default()
        //     .with_device_id(0)
        //     .with_arena_extend_strategy(ort::ArenaExtendStrategy::NextPowerOfTwo)
        //     .with_memory_limit(2 * 1024 * 1024 * 1024)
        //     .with_conv_algorithm_search(ort::CUDAExecutionProviderCuDNNConvAlgoSearch::Exhaustive)
        //     .with_copy_in_default_stream(true);
        // let decoder_dm = DirectMLExecutionProvider::default().with_device_id(2);

        // if !ort::ExecutionProvider::is_available(&cuda)? {
        //     anyhow::bail!("Please compile ONNX Runtime with CUDA!")
        // }

        // ort::ExecutionProvider::register(&cuda, &builder).map_err(|v| {
        //     anyhow::anyhow!("Please check if ONNX Runtime is compiled with CUDA support: {v}")
        // })?;
        // println!("CUDA:{:?} DirectML:{:?}", encoder_cuda.is_available().unwrap(), decoder_dm.is_available().unwrap());
        let encoder_session = encoder_builder
            .with_execution_providers(
                [
                    // encoder_cuda.build(),
                    // dm.build()
                ]
            )?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(8)?
            .with_inter_threads(8)?
            .commit_from_memory(ENCODER_BYTES)?;
        let decoder_session = decoder_builder
            .with_execution_providers(
                [
                    // decoder_cuda.build(),
                    // decoder_dm.build(),
                    // dm.build()
                ]
            )?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            // .with_parallel_execution(true)?
            .with_intra_threads(12)?
            .with_inter_threads(12)?
            .commit_from_memory(DECODER_BYTES)?;
        Ok(MixTexOnnx {
            encoder_session,
            decoder_session,
            tokenizer: Tokenizer::from_str(TOKENIZER_STR).expect("Fail to load tokenizer"),
        })
    }

    fn init_decode(&self, img: &[f32])-> std::result::Result<(usize,Array<f32, IxDyn>,SessionOutputs), Box<dyn std::error::Error>>{

        let encoder_result = self.encoder_session.run(ort::inputs! {"pixel_values" => ([1,3,448,448],img)}?)?;
        let hidden_state = encoder_result["last_hidden_state"].try_extract_tensor::<f32>()?.to_owned();
        let mut decode_input_ids = array![[0,0,30000_i64]];
        let  k_0 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        let  k_1 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        let  k_2 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        let  v_0 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        let  v_1 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        let  v_2 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        // (1, 2, 3).f();

        // eprintln!("Encode end, start decoder loop");

        // let check_rate = MAX_LENGTH / 64;

        let decoder_result = self.decoder_session.run(ort::inputs! {
            "encoder_hidden_states" => hidden_state.view(),
            "input_ids"=> decode_input_ids.view(),
            "use_cache_branch"=>array![true],
            "past_key_values.0.key"=>k_0.view(),
            "past_key_values.0.value"=>v_0.view(),
            "past_key_values.1.key"=>k_1.view(),
            "past_key_values.1.value"=>v_1.view(),
            "past_key_values.2.key"=>k_2.view(),
            "past_key_values.2.value"=>v_2.view(),
            }?)?;

        let mut logits = decoder_result["logits"].try_extract_tensor::<f32>()?;
        let mut next_token_id = logits.slice(s![0,-1,..])
            .iter()
            .enumerate()
            .max_by(|&(_, x), &(_, y)| {
                x.partial_cmp(&y).unwrap()
            })
            .unwrap()
            .0;

        Ok((next_token_id,hidden_state,decoder_result))
    }

    fn decode_once<'a>(&'a self,state:(usize,Array<f32, IxDyn>,SessionOutputs<'a,'a>))-> std::result::Result<(usize,Array<f32, IxDyn>,SessionOutputs<'a,'a>), Box<dyn std::error::Error>>{
        let (mut next_token_id,hidden_state,mut decoder_result) = state;
        decoder_result = self.decoder_session.run(ort::inputs! {
            "encoder_hidden_states" => hidden_state.view(),
            "input_ids"=> array![[next_token_id as i64]],
            "use_cache_branch"=>array![true],
            "past_key_values.0.key"=>decoder_result["present.0.key"].try_extract_tensor::<f32>()?,
            "past_key_values.0.value"=>decoder_result["present.0.value"].try_extract_tensor::<f32>()?,
            "past_key_values.1.key"=>decoder_result["present.1.key"].try_extract_tensor::<f32>()?,
            "past_key_values.1.value"=>decoder_result["present.1.value"].try_extract_tensor::<f32>()?,
            "past_key_values.2.key"=>decoder_result["present.2.key"].try_extract_tensor::<f32>()?,
            "past_key_values.2.value"=>decoder_result["present.2.value"].try_extract_tensor::<f32>()?,
            }?)?;
        // println!("---->loop {i} {:?} ",start_loop.elapsed());
        let logits = decoder_result["logits"].try_extract_tensor::<f32>()?;
        next_token_id = logits.slice(s![0,-1,..])
            .iter()
            .enumerate()
            .max_by(|&(_, x), &(_, y)| {
                x.partial_cmp(&y).unwrap()
            })
            .unwrap()
            .0;
        Ok((next_token_id, hidden_state, decoder_result))
    }

    pub fn inference(&self, img: &[f32]) -> std::result::Result<String, Box<dyn std::error::Error>> {
        // eprintln!("Start inference!");
        let start = std::time::Instant::now();
        // eprintln!("Encode end, start decoder loop");

        let check_rate = MAX_LENGTH / 64;
        let mut result_idx = [0_u32; MAX_LENGTH];


        let (mut next_token_id,mut hidden_state,mut decoder_result) = self.init_decode(img)?;
        result_idx[0] = next_token_id as u32;

        for i in 1..MAX_LENGTH {
            // let start_loop = std::time::Instant::now();
            (next_token_id,hidden_state,decoder_result) = self.decode_once((next_token_id,hidden_state,decoder_result))?;
            result_idx[i] = next_token_id as u32;

            // stop token 的id，这里硬编码
            if next_token_id == 30000 {
                break;
            }
            // decode_input_ids = concatenate![Axis(1),decode_input_ids,array![[next_token_id as i64]]];
            if ((i + 1) % check_rate == 0) && check_repeat(&result_idx[..=i]) {
                break;
            }
        }
        eprintln!("\x1b[31mTime cost:\x1b[32m{:?}\x1b[0m", start.elapsed());

        Ok(self.tokenizer.decode(&result_idx, true).unwrap())
    }
    pub fn inference_by_step<F>(&self, img: &[f32],mut callback:F)-> std::result::Result<String, Box<dyn std::error::Error>>
    where
        F: FnMut(String)->bool
    {
        let check_rate = MAX_LENGTH / 64;
        let mut result_idx = [0_u32; MAX_LENGTH];
        let mut result_string = String::with_capacity(512);


        let (mut next_token_id,mut hidden_state,mut decoder_result) = self.init_decode(img)?;
        result_idx[0] = next_token_id as u32;
        let res  = self.tokenizer.decode(&[next_token_id as u32],true).unwrap();
        result_string+=&res;
        callback(res);

        for i in 1..MAX_LENGTH {
            (next_token_id,hidden_state,decoder_result) = self.decode_once((next_token_id,hidden_state,decoder_result))?;
            let res  = self.tokenizer.decode(&[next_token_id as u32],true).unwrap();
            result_idx[i] = next_token_id as u32;
            result_string+=&res;
            if callback(res){
                break
            }

            // stop token 的id，这里硬编码
            if next_token_id == 30000 {
                break;
            }
            if ((i + 1) % check_rate == 0) && check_repeat(&result_idx[..=i]) {
                break;
            }
        }

        Ok(result_string)
    }
}