use log::{error, info};
use ndarray::prelude::*;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::{Session, SessionOutputs};
use std::str::FromStr;
use tokenizers::Tokenizer;

use crate::mixtex::{check_repeat, OcrModel};
use crate::model::{DECODER_BYTES, ENCODER_BYTES, TOKENIZER_STR};
use anyhow::{anyhow, Result};
use ort::execution_providers::ExecutionProvider;
use ort::io_binding::IoBinding;
use ort::memory::{AllocationDevice, Allocator, AllocatorType, MemoryInfo, MemoryType};
use ort::value::Tensor;

const MAX_LENGTH: usize = 512;
const STOP_TOKEN_IDX: usize = 30000;
pub struct MixTexOnnx {
    encoder_session: Session,
    decoder_session: Session,
    tokenizer: Tokenizer,
}

impl MixTexOnnx {
    fn prefill(&self, img: &[f32]) -> Result<(usize, IoBinding)> {
        let mut encoder_io = self.encoder_session.create_binding()?;

        // #[cfg(windows)]
        // let encoder_allocator = Allocator::new(
        //     &self.encoder_session,
        //     MemoryInfo::new(
        //         AllocationDevice::DIRECTML,
        //         0,
        //         AllocatorType::Device,
        //         MemoryType::CPUInput,
        //     )?,
        // )?;
        // #[cfg(any(target_os = "linux", target_os = "macos"))]
        // let encoder_allocator = Allocator::new(
        //     &self.encoder_session,
        //     #[cfg(windows)]
        //     MemoryInfo::new(
        //         AllocationDevice::CPU,
        //         0,
        //         AllocatorType::Device,
        //         MemoryType::CPUInput,
        //     )?,
        // )?;

        let input: Tensor<f32> =
            Tensor::from_array(([1, 3, 448, 448], img.to_vec().into_boxed_slice()))?;

        encoder_io.bind_input("pixel_values", &input)?;
        encoder_io.bind_output(
            "last_hidden_state",
            Tensor::<f32>::new(self.encoder_session.allocator(), [1, 196, 768])?,
        )?;
        let mut encoder_result = match encoder_io.run() {
            Ok(x) => x,
            Err(e) => {
                self.encoder_session.end_profiling().unwrap();
                return Err(anyhow!("{e:?}"));
            }
        };
        let hidden_state = encoder_result.remove("last_hidden_state").unwrap();
        // let decode_input_ids = array![[0, 0, 30000_i64]];
        // let k_0 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        // let k_1 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        // let k_2 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        // let v_0 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        // let v_1 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();
        // let v_2 = Array::<f32, _>::zeros((1, 12, 0, 64).f()).into_dyn();

        // #[cfg(windows)]
        // let decoder_allocator = Allocator::new(
        //     &self.decoder_session,
        //     MemoryInfo::new(
        //         AllocationDevice::DIRECTML,
        //         0,
        //         AllocatorType::Device,
        //         MemoryType::CPUInput,
        //     )?,
        // )?;
        // #[cfg(any(target_os = "linux", target_os = "macos"))]
        // let decoder_allocator = Allocator::new(
        //     &self.decoder_session,
        //     #[cfg(windows)]
        //     MemoryInfo::new(
        //         AllocationDevice::CPU,
        //         0,
        //         AllocatorType::Device,
        //         MemoryType::CPUInput,
        //     )?,
        // )?;

        let mut decoder_io = self.decoder_session.create_binding()?;
        decoder_io.bind_input("encoder_hidden_states", &hidden_state)?;
        decoder_io.bind_input(
            "input_ids",
            &Tensor::<i64>::from_array(([1, 3], vec![0, 0, 30000_i64]))?,
        )?;
        #[cfg(windows)]
        decoder_io.bind_output(
            "logits",
            Tensor::<f32>::new(self.decoder_session.allocator(), [1, 3, 30002])?,
        )?;
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        decoder_io.bind_output(
            "logits",
            Tensor::<f32>::new(self.decoder_session.allocator(), [1, 3, 30002])?,
        )?;

        let fake_kv = Tensor::from_array(Array4::<f32>::zeros((1, 12, 0, 64)))?;
        for i in 0..3 {
            decoder_io.bind_input(&format!("past_key_values.{i}.value"), &fake_kv)?;
            decoder_io.bind_input(&format!("past_key_values.{i}.key"), &fake_kv)?;
            decoder_io.bind_output(
                &format!("present.{i}.key"),
                Tensor::<f32>::new(self.decoder_session.allocator(), [1, 12, 3, 64])?,
            )?;
            decoder_io.bind_output(
                &format!("present.{i}.value"),
                Tensor::<f32>::new(self.decoder_session.allocator(), [1, 12, 3, 64])?,
            )?;
        }
        let mut next_token_id;
        let mut kv_cache = Vec::with_capacity(6);
        {
            let mut decoder_result = decoder_io.run()?;

            let logits = decoder_result
                .remove("logits")
                .unwrap()
                .try_extract_tensor::<f32>()?
                .to_owned();
            next_token_id = logits
                .slice(s![0, -1, ..])
                .iter()
                .enumerate()
                .max_by(|&(_, x), &(_, y)| x.partial_cmp(&y).unwrap())
                .unwrap()
                .0;

            for i in 0..3 {
                kv_cache.push(
                    decoder_result
                        .remove(&format!("present.{i}.value"))
                        .unwrap(),
                );
                kv_cache.push(decoder_result.remove(&format!("present.{i}.key")).unwrap());
            }
        }
        decoder_io.clear_outputs();

        for i in 0..3 {
            decoder_io.bind_input(&format!("past_key_values.{i}.value"), &kv_cache[i * 2])?;
            decoder_io.bind_input(&format!("past_key_values.{i}.key"), &kv_cache[i * 2 + 1])?;
            decoder_io.bind_output(
                &format!("present.{i}.key"),
                Tensor::<f32>::new(self.decoder_session.allocator(), [1, 12, 4, 64])?,
            )?;
            decoder_io.bind_output(
                &format!("present.{i}.value"),
                Tensor::<f32>::new(self.decoder_session.allocator(), [1, 12, 4, 64])?,
            )?;
        }

        #[cfg(windows)]
        decoder_io.bind_output(
            "logits",
            Tensor::<f32>::new(self.decoder_session.allocator(), [1, 1, 30002])?,
        )?;
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        decoder_io.bind_output(
            "logits",
            Tensor::<f32>::new(self.decoder_session.allocator(), [1, 1, 30002])?,
        )?;
        Ok((next_token_id, decoder_io))
    }

    fn decode_once(&self, state: (usize, IoBinding, usize)) -> Result<(usize, IoBinding)> {
        let (mut next_token_id, mut decoder_io, num_tokens) = state;
        decoder_io.bind_input(
            "input_ids",
            &Tensor::<i64>::from_array(([1, 1], vec![next_token_id as i64]))?,
        )?;

        let mut kv_cache = Vec::with_capacity(6);

        {
            let mut decoder_result = decoder_io.run()?;

            let logits = decoder_result
                .remove("logits")
                .unwrap()
                .try_extract_tensor::<f32>()?
                .to_owned();
            next_token_id = logits
                .slice(s![0, -1, ..])
                .iter()
                .enumerate()
                .max_by(|&(_, x), &(_, y)| x.partial_cmp(&y).unwrap())
                .unwrap()
                .0;

            for i in 0..3 {
                kv_cache.push(
                    decoder_result
                        .remove(&format!("present.{i}.value"))
                        .unwrap(),
                );
                kv_cache.push(decoder_result.remove(&format!("present.{i}.key")).unwrap());
            }
        }
        decoder_io.clear_outputs();
        // let num_tokens = kv_cache[0].shape()?[2];

        for i in 0..3 {
            decoder_io.bind_input(&format!("past_key_values.{i}.value"), &kv_cache[i * 2])?;
            decoder_io.bind_input(&format!("past_key_values.{i}.key"), &kv_cache[i * 2 + 1])?;

            decoder_io.bind_output(
                &format!("present.{i}.key"),
                Tensor::<f32>::new(
                    self.decoder_session.allocator(),
                    [1, 12, num_tokens + 1, 64],
                )?,
            )?;
            decoder_io.bind_output(
                &format!("present.{i}.value"),
                Tensor::<f32>::new(
                    self.decoder_session.allocator(),
                    [1, 12, num_tokens + 1, 64],
                )?,
            )?;
        }

        #[cfg(windows)]
        decoder_io.bind_output(
            "logits",
            Tensor::<f32>::new(self.decoder_session.allocator(), [1, 1, 30002])?,
        )?;
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        decoder_io.bind_output(
            "logits",
            Tensor::<f32>::new(self.decoder_session.allocator(), [1, 1, 30002])?,
        )?;

        Ok((next_token_id, decoder_io))
    }
}

impl OcrModel for MixTexOnnx {
    fn build() -> Result<Self> {
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
        println!(
            "CUDA:{:?} DirectML:{:?}",
            false,
            ort::execution_providers::DirectMLExecutionProvider::default()
                .is_available()
                .unwrap()
        );
        let encoder_session = encoder_builder
            .with_execution_providers([
                #[cfg(windows)]
                ort::execution_providers::DirectMLExecutionProvider::default()
                    .with_device_id(1)
                    .build(),
                ort::execution_providers::OneDNNExecutionProvider::default().with_use_arena(true).build()
            ])?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            // .with_memory_pattern(true)?
            // .with_qdq_cleanup()?
            // .with_parallel_execution(true)?
            // .with_intra_threads(8)?
            // .with_inter_threads(8)?
            // .with_profiling(
            //     r#"C:\Users\whl\WorkSpace\RustProjects\GotOnnx\profile\mixtex_encoder"#,
            // )?
            .with_profiling(
                std::env::current_exe()?
                    .parent()
                    .unwrap()
                    .join("mixtex_encoder_profile"),
            )?
            .commit_from_memory(ENCODER_BYTES)?;
        let decoder_session = decoder_builder
            .with_execution_providers([
                // decoder_cuda.build(),
                // decoder_dm.build(),
                // dm.build()
                #[cfg(windows)]
                ort::execution_providers::DirectMLExecutionProvider::default()
                    // .with_device_id(1)
                    .build(),
                ort::execution_providers::OneDNNExecutionProvider::default().with_use_arena(true).build()
            ])?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_profiling(
                std::env::current_exe()?
                    .parent()
                    .unwrap()
                    .join("mixtex_decoder_profile"),
            )?
            // .with_profiling(
            //     r#"C:\Users\whl\WorkSpace\RustProjects\GotOnnx\profile\mixtex_decoder"#,
            // )?
            // .with_qdq_cleanup()?
            // .with_parallel_execution(true)?
            // .with_intra_threads(12)?
            // .with_inter_threads(12)?
            .commit_from_memory(DECODER_BYTES)?;
        Ok(MixTexOnnx {
            encoder_session,
            decoder_session,
            tokenizer: Tokenizer::from_str(TOKENIZER_STR).expect("Fail to load tokenizer"),
        })
    }
    fn inference(&self, img: &[f32]) -> Result<String> {
        // eprintln!("Start inference!");
        let start = std::time::Instant::now();
        // eprintln!("Encode end, start decoder loop");

        let check_rate = MAX_LENGTH / 64;
        let mut result_idx = [0_u32; MAX_LENGTH];

        let (mut next_token_id, mut decoder_io) = self.prefill(img)?;
        result_idx[0] = next_token_id as u32;

        for i in 1..MAX_LENGTH {
            // let start_loop = std::time::Instant::now();
            (next_token_id, decoder_io) = self.decode_once((next_token_id, decoder_io, i + 3))?;
            result_idx[i] = next_token_id as u32;

            // stop token 的id
            if next_token_id == STOP_TOKEN_IDX {
                break;
            }
            // decode_input_ids = concatenate![Axis(1),decode_input_ids,array![[next_token_id as i64]]];
            if ((i + 1) % check_rate == 0) && check_repeat(&result_idx[..=i]) {
                break;
            }
        }
        info!("\x1b[31mTime cost:\x1b[32m{:?}\x1b[0m", start.elapsed());

        Ok(self.tokenizer.decode(&result_idx, true).unwrap())
    }
    fn generate<F>(&self, img: &[f32], mut callback: F) -> Result<String>
    where
        F: FnMut(String) -> bool,
    {
        let check_rate = MAX_LENGTH / 64;
        let mut result_idx = [0_u32; MAX_LENGTH];
        let mut result_string = String::with_capacity(512);

        let (mut next_token_id, mut decoder_io) = self.prefill(img)?;
        result_idx[0] = next_token_id as u32;
        let res = self
            .tokenizer
            .decode(&[next_token_id as u32], true)
            .unwrap();
        result_string += &res;
        callback(res);

        for i in 1..MAX_LENGTH {
            (next_token_id, decoder_io) = self.decode_once((next_token_id, decoder_io, i + 3))?;
            let res = self
                .tokenizer
                .decode(&[next_token_id as u32], true)
                .unwrap();
            result_idx[i] = next_token_id as u32;
            result_string += &res;
            if callback(res) {
                break;
            }

            // stop token 的id，这里硬编码
            if next_token_id == STOP_TOKEN_IDX {
                break;
            }
            if ((i + 1) % check_rate == 0) && check_repeat(&result_idx[..=i]) {
                break;
            }
        }
        self.decoder_session.end_profiling()?;
        self.encoder_session.end_profiling()?;
        Ok(result_string)
    }
}
