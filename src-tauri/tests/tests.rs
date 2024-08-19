
#[cfg(test)]
mod test{
    use mixtex_rs_gui::*;

    #[test]
    fn test_image_pre(){
        use vit_image_processor::{padding,preprocess};
        preprocess(r"C:\Users\whl\RustProjects\MixTex-rs-GUI-new\tests\test.png").unwrap();
    }

}