use anyhow::{anyhow, Result};
use hassle_rs::{Dxc, DxcCompiler, DxcIncludeHandler, DxcLibrary};
use std::path::PathBuf;

fn read_source(path: &str) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

struct IncludeHandler {}

impl IncludeHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl DxcIncludeHandler for IncludeHandler {
    fn load_source(&mut self, path: String) -> Option<String> {
        read_source(&path).ok()
    }
}

#[allow(dead_code)]
pub struct ShaderCompiler {
    library: DxcLibrary,
    compiler: DxcCompiler,
    dxc: Dxc,
}

pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl ShaderStage {
    pub fn profile_name(&self) -> &'static str {
        match self {
            ShaderStage::Vertex => "vs_6_0",
            ShaderStage::Fragment => "ps_6_0",
            ShaderStage::Compute => "cs_6_0",
        }
    }

    pub fn entry_point(&self) -> &'static str {
        match self {
            ShaderStage::Vertex => "vs_main",
            ShaderStage::Fragment => "ps_main",
            ShaderStage::Compute => "cs_main",
        }
    }
}

impl ShaderCompiler {
    pub fn new() -> Self {
        let dxc = Dxc::new(Some(PathBuf::from("dxc"))).unwrap();
        let compiler = dxc.create_compiler().unwrap();
        let library = dxc.create_library().unwrap();

        Self {
            dxc,
            compiler,
            library,
        }
    }

    fn compile(
        &self,
        path: &str,
        stage: ShaderStage,
    ) -> Result<Vec<u8>> {
        let source = read_source(path)?;

        let blob = self
            .library
            .create_blob_with_encoding_from_str(&source)
            .unwrap();

        let profile = stage.profile_name();
        let entry_point = stage.entry_point();
        let args = ["-HV 2021", "-I /", "-spirv"].as_slice();
        let mut include_handler = IncludeHandler::new();
        let defines = &[];
        let result = self.compiler.compile(
            &blob,
            path,
            entry_point,
            profile,
            args,
            Some(&mut include_handler),
            defines,
        );

        match result {
            Ok(v) => Ok(v.get_result().unwrap().to_vec()),
            Err(v) => {
                let message = self
                    .library
                    .get_blob_as_string(&v.0.get_error_buffer().unwrap().into())?;
                Err(anyhow!("shader error ({path}): {message}"))
            }
        }
    }
}

fn main() {
    let compiler = ShaderCompiler::new();

    let spirv = compiler.compile("kernel/kernel.hlsl", ShaderStage::Compute);
    let _spirv = match spirv {
        Ok(spirv) => spirv,
        Err(e) => {
            eprintln!("couldn't compile shader:");
            eprintln!("{e}");
            return;
        }
    };

    println!("Hello, world!");
}
