use std::path::Path;
use tokio::fs::File;
use tokio::prelude::*;

pub struct ShaderCache {
    compiler: shaderc::Compiler,
}

impl ShaderCache {
    pub fn new() -> Self {
        Self {
            compiler: shaderc::Compiler::new().unwrap(),
        }
    }

    pub async fn get_shader<P: AsRef<Path>>(
        &mut self,
        path: P,
        shader_kind: shaderc::ShaderKind,
    ) -> Vec<u32> {
        let path = path.as_ref();
        let input_file_name = path
            .file_name()
            .expect("Expected path to have a filename")
            .to_str()
            .expect("Expected filename to be valid unicode");

        let mut source_text = Vec::new();
        let mut file = File::open(path)
            .await
            .expect("Failed to open shader source file");
        file.read_to_end(&mut source_text)
            .await
            .expect("Failed to read shader source file");
        let source_text =
            std::str::from_utf8(&source_text).expect("Expected shader source to be valid utf8");

        let entry_point_name = "main";
        let additional_options = None;
        self.compiler
            .compile_into_spirv(
                &source_text,
                shader_kind,
                input_file_name,
                entry_point_name,
                additional_options,
            )
            .expect("Failed to compile shader source")
            .as_binary()
            .to_vec()
    }
}
