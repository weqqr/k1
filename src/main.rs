use anyhow::{anyhow, Result};
use hassle_rs::{Dxc, DxcCompiler, DxcIncludeHandler, DxcLibrary};
use pollster::FutureExt;
use std::borrow::Cow;
use std::path::PathBuf;
use wgpu::{ComputePipelineDescriptor, InstanceDescriptor, RequestAdapterOptions};

const WORK_GROUP_SIZE: [u32; 2] = [8, 8];

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

    fn compile(&self, path: &str, stage: ShaderStage) -> Result<Vec<u8>> {
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
                Err(anyhow!("shader error ({path}):\n{message}"))
            }
        }
    }
}

pub struct Renderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    fn new() -> Self {
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .block_on()
            .unwrap();

        let features = wgpu::Features::default() | wgpu::Features::SPIRV_SHADER_PASSTHROUGH;
        let limits = wgpu::Limits::default();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features,
                    limits,
                },
                None,
            )
            .block_on()
            .unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    fn render(&self, pass: &PathTracingPass) {
        let mut cmd = self.device.create_command_encoder(&Default::default());

        pass.execute(&mut cmd);

        self.queue.submit([cmd.finish()]);
    }
}

pub struct PathTracingPass {
    layout: wgpu::PipelineLayout,
    pipeline: wgpu::ComputePipeline,
}

impl PathTracingPass {
    fn new(kernel: &[u32], device: &wgpu::Device) -> Self {
        let module = unsafe {
            device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                label: None,
                source: Cow::Borrowed(kernel),
            })
        };

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &module,
            entry_point: ShaderStage::Compute.entry_point(),
        });

        Self { layout, pipeline }
    }

    fn execute(&self, cmd: &mut wgpu::CommandEncoder) {
        let mut pass = cmd.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

        pass.set_pipeline(&self.pipeline);

        let x_groups = 1280 / WORK_GROUP_SIZE[0];
        let y_groups = 720 / WORK_GROUP_SIZE[1];

        pass.dispatch_workgroups(x_groups, y_groups, 1);
    }
}

fn main() {
    let compiler = ShaderCompiler::new();

    let kernel = compiler.compile("kernel/kernel.hlsl", ShaderStage::Compute);
    let kernel = match kernel {
        Ok(spirv) => spirv,
        Err(e) => {
            eprintln!("couldn't compile shader:");
            eprintln!("{e}");
            return;
        }
    };

    let renderer = Renderer::new();

    let path_tracing_pass = PathTracingPass::new(bytemuck::cast_slice(&kernel), &renderer.device);

    renderer.render(&path_tracing_pass);

    println!("Hello, world!");
}
