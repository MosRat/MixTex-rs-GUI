use std::str::FromStr;

use tokenizers::Tokenizer;
use windows::{
    AI::MachineLearning::*,
    core::*,
    Win32::System::WinRT::*,
};
use windows::Foundation::IMemoryBufferReference;
use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream, IOutputStream, RandomAccessStreamReference};

use crate::{check_repeat, DECODER_BYTES, ENCODER_BYTES, TOKENIZER_STR};

const MAX_LENGTH: usize = 512;

pub enum WinMLDeviceType {
    Cpu,
    DirectML,
}

#[allow(dead_code)]
pub struct MixTexWinML {
    encoder_model: LearningModel,
    decoder_model: LearningModel,
    encoder_session: LearningModelSession,
    decoder_session: LearningModelSession,
    tokenizers: Tokenizer,
}

impl MixTexWinML {
    pub fn build(
        device_type: Option<WinMLDeviceType>,
    ) -> Self {
        // let encoder_path = HSTRING::from(PathBuf::from(model_path).join("encoder_model.onnx").to_str().expect("Model path error"));
        // let decoder_path = HSTRING::from(PathBuf::from(model_path).join("decoder_model_merged.onnx").to_str().expect("Model path error"));
        // let tokenizer_path = PathBuf::from(model_path).join("tokenizer/tokenizer.json").into_os_string().into_string().expect("Tokenizer path error");

        let device = LearningModelDevice::Create(
            match device_type {
                None => { LearningModelDeviceKind::Default }
                Some(d) => {
                    match d {
                        WinMLDeviceType::Cpu => LearningModelDeviceKind::Cpu,
                        WinMLDeviceType::DirectML => LearningModelDeviceKind::DirectXHighPerformance
                    }
                }
            }
        ).expect("Create device error");
        let dm_device = LearningModelDevice::Create(LearningModelDeviceKind::DirectXHighPerformance).unwrap();

        let encoder_model = LearningModel::LoadFromStream(&RandomAccessStreamReference::CreateFromStream(&bytes_to_stream(ENCODER_BYTES).unwrap()).unwrap())
            .expect("Load model fail!");
        let decoder_model = LearningModel::LoadFromStream(&RandomAccessStreamReference::CreateFromStream(&bytes_to_stream(DECODER_BYTES).unwrap()).unwrap())
            .expect("Load model fail!");

        decoder_model.InputFeatures().expect("xxx");
        decoder_model.Metadata().expect("xxx");
        decoder_model.Name().expect("xxx");
        // let encoder_model = LearningModel::LoadFromFilePath(&encoder_path).expect("Load model fail!");
        // let decoder_model = LearningModel::LoadFromFilePath(&decoder_path).expect("Load model fail!");

        let encoder_session = LearningModelSession::CreateFromModelOnDevice(&encoder_model, &device).expect("Create session fail");
        let decoder_session = LearningModelSession::CreateFromModelOnDevice(&decoder_model, &dm_device).expect("Create session fail");

        MixTexWinML {
            encoder_model,
            decoder_model,
            encoder_session,
            decoder_session,
            tokenizers: Tokenizer::from_str(TOKENIZER_STR).expect("Fail to load tokenizer"),
        }
    }

    pub fn inference(&self, img: &[f32]) -> std::result::Result<String, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();


        // encoder prepare
        let binding = LearningModelBinding::CreateFromSession(&self.encoder_session)?;
        let input_tensor = TensorFloat::CreateFromShapeArrayAndDataArray(&[1, 3, 448, 448], img)?;
        binding.Bind(&HSTRING::from("pixel_values"), &input_tensor).expect("Bind input fail!");

        // encoder inference
        let hidden_stat: TensorFloat = self.encoder_session.Evaluate(&binding, &HSTRING::from("run_id"))?
            .Outputs()?
            .Lookup(&HSTRING::from("last_hidden_state"))?
            .cast()?;

        // eprintln!("Encoder time cost:{:?}", start.elapsed());

        // decoder prepare
        // tokenizer("<s>", return_tensors="np").input_ids = [[0, 0, 30000]]
        let mut decode_input_ids = TensorInt64Bit::CreateFromShapeArrayAndDataArray(&[1, 3], &[0, 0, 30000])?;
        let fill_tensor = TensorFloat::CreateFromShapeArrayAndDataArray(&[1, 12, 0, 64], &[])?;
        // let mut result_text = String::with_capacity(100);
        let mut result_idx = [0_u32; MAX_LENGTH];
        let check_rate = MAX_LENGTH / 64;

        // Prefill输入
        let binding = LearningModelBinding::CreateFromSession(&self.decoder_session)?;
        // bing input
        binding.Bind(&HSTRING::from("encoder_hidden_states"), &hidden_stat).expect("Bind input fail!");
        binding.Bind(&HSTRING::from("input_ids"), &decode_input_ids).expect("Bind input fail!");


        // only for fill inputs
        binding.Bind(&HSTRING::from("use_cache_branch"), &TensorBoolean::CreateFromShapeArrayAndDataArray(&[1], &[true])?).expect("Bind input fail!");
        binding.Bind(&HSTRING::from("past_key_values.0.key"), &fill_tensor).expect("Bind input fail!");
        binding.Bind(&HSTRING::from("past_key_values.0.value"), &fill_tensor).expect("Bind input fail!");
        binding.Bind(&HSTRING::from("past_key_values.1.key"), &fill_tensor).expect("Bind input fail!");
        binding.Bind(&HSTRING::from("past_key_values.1.value"), &fill_tensor).expect("Bind input fail!");
        binding.Bind(&HSTRING::from("past_key_values.2.key"), &fill_tensor).expect("Bind input fail!");
        binding.Bind(&HSTRING::from("past_key_values.2.value"), &fill_tensor).expect("Bind input fail!");
        // binding.
        // eprintln!("Decoder loop {i} binding time cost {:?}", loop_start.elapsed());
        // let inference_start = std::time::Instant::now();

        // post-process outputs
        let mut result = self.decoder_session.Evaluate(&binding, &HSTRING::from("run_id"))?
            .Outputs()?;
        let mut logits : TensorFloat = result
            .Lookup(&HSTRING::from("logits"))?
            .cast()?;

        let buffer = logits.CreateReference()?;
        let ptr;
        unsafe {
            ptr = buffer_get_at::<f32>(buffer);
        }


        let mut next_token_id = (0..30002).map(
            |i| {
                unsafe {
                    *ptr.add(60004 + i)
                }
                // buffer.
                //     buffer.GetAt((length + i) as u32).unwrap()
            }
        )
            .enumerate()
            .max_by(|&(_, x), &(_, y)| {
                x.partial_cmp(&y).unwrap()
            })
            .unwrap()
            .0;
        result_idx[0] = next_token_id as u32;



        // decoder inference loop
        for i in 1..MAX_LENGTH {
            // eprintln!("-------start {i} -------");
            // let loop_start = std::time::Instant::now();
            let binding = LearningModelBinding::CreateFromSession(&self.decoder_session)?;
            // bing input
            binding.Bind(&HSTRING::from("encoder_hidden_states"), &hidden_stat).expect("Bind input fail!");
            binding.Bind(&HSTRING::from("input_ids"), &TensorInt64Bit::CreateFromShapeArrayAndDataArray(&[1,1],&[next_token_id as i64])?).expect("Bind input fail!");


            // only for fill inputs
            binding.Bind(&HSTRING::from("use_cache_branch"), &TensorBoolean::CreateFromShapeArrayAndDataArray(&[1], &[true])?).expect("Bind input fail!");
            binding.Bind(&HSTRING::from("past_key_values.0.key"), &result.Lookup(&HSTRING::from("present.0.key"))?).expect("Bind input fail!");
            binding.Bind(&HSTRING::from("past_key_values.0.value"), &result.Lookup(&HSTRING::from("present.0.value"))?).expect("Bind input fail!");
            binding.Bind(&HSTRING::from("past_key_values.1.key"), &result.Lookup(&HSTRING::from("present.1.key"))?).expect("Bind input fail!");
            binding.Bind(&HSTRING::from("past_key_values.1.value"), &result.Lookup(&HSTRING::from("present.1.value"))?).expect("Bind input fail!");
            binding.Bind(&HSTRING::from("past_key_values.2.key"), &result.Lookup(&HSTRING::from("present.2.key"))?).expect("Bind input fail!");
            binding.Bind(&HSTRING::from("past_key_values.2.value"), &result.Lookup(&HSTRING::from("present.2.value"))?).expect("Bind input fail!");
            // binding.
            // eprintln!("Decoder loop {i} binding time cost {:?}", loop_start.elapsed());
            // let inference_start = std::time::Instant::now();

            // post-process outputs
            result = self.decoder_session.Evaluate(&binding, &HSTRING::from("run_id"))?
                .Outputs()?;
            logits = result
                .Lookup(&HSTRING::from("logits"))?
                .cast()?;
            // eprintln!("{:?}",logits.Shape()?.into_iter().collect::<Vec<i64>>());
            // eprintln!("Decoder loop {i} inference time cost {:?}", inference_start.elapsed());
            // let pp_start = std::time::Instant::now();


            let buffer = logits.CreateReference()?;
            let ptr;
            unsafe {
                ptr = buffer_get_at::<f32>(buffer);
            }


            next_token_id = (0..30002).map(
                |i| {
                    unsafe {
                        *ptr.add(i)
                    }
                }
            )
                .enumerate()
                .max_by(|&(_, x), &(_, y)| {
                    x.partial_cmp(&y).unwrap()
                })
                .unwrap()
                .0;

            // println!("Decoder {i} deal result time cost :{:?}", pp_start.elapsed());

            // result_text += &self.tokenizers.decode(&[next_token_id as u32], true).expect("Tokenizer decode fail!");
            result_idx[i] = next_token_id as u32;
            // println!("Decoder {i} deal tokenizer time cost :{:?}", pp_start.elapsed());
            // println!("{result_text}");

            if next_token_id == 30000 {
                break;
            }

            if ((i + 1) % check_rate == 0) && check_repeat(&result_idx[..=i]) {
                break
            }

            // 构造下一次输入
            // let next_start = std::time::Instant::now();
            let tmp: Vec<i64> = decode_input_ids
                .GetAsVectorView()?
                .into_iter()
                .chain(std::iter::once(next_token_id as i64))
                .collect();
            decode_input_ids = TensorInt64Bit::CreateFromShapeArrayAndDataArray(&[1, (i + 4) as i64], &tmp).unwrap();

            // eprintln!("Decoder loop {i} next idx time cost {:?}", next_start.elapsed());
            // eprintln!("Decoder loop {i} total time cost {:?}", loop_start.elapsed());
        }


        eprintln!("\x1b[31mTime cost:\x1b[32m{:?}\x1b[0m", start.elapsed());

        Ok(self.tokenizers.decode(&result_idx, true).unwrap())
    }
}

pub unsafe fn buffer_get_at<T: Copy>(buffer: IMemoryBufferReference) -> *mut T {
    let mut raw_ptr: *mut u8 = std::ptr::null_mut();
    let mut capacity: u32 = 0;
    let buffer_access:IMemoryBufferByteAccess = buffer.cast().unwrap();

    buffer_access.GetBuffer(&mut raw_ptr as *mut *mut u8, &mut capacity as *mut u32).unwrap();
    let t_ptr: *mut T = raw_ptr as *mut T;
    t_ptr
}

pub fn bytes_to_stream(bytes: &[u8]) ->std::result::Result<InMemoryRandomAccessStream,Box<dyn std::error::Error>>{
    let stream = InMemoryRandomAccessStream::new()?;
    let output_stream: IOutputStream = stream.GetOutputStreamAt(0).unwrap();

    let writer = DataWriter::CreateDataWriter(&output_stream)?;
    writer.WriteBytes(bytes)?;

    writer.StoreAsync()?.get()?;
    writer.FlushAsync()?.get()?;

    Ok(stream)
}