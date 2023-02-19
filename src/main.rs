use anyhow::{anyhow, Result};
use hassle_rs::{Dxc, DxcCompiler, DxcIncludeHandler, DxcLibrary};
use pollster::FutureExt;
use std::borrow::Cow;
use std::fs::File;
use std::io::BufWriter;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::time::Duration;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, ComputePipelineDescriptor, InstanceDescriptor, RequestAdapterOptions,
};

const OUTPUT_SIZE: [u32; 2] = [1280, 720];
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

    output: wgpu::Texture,
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

        let output = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: OUTPUT_SIZE[0],
                height: OUTPUT_SIZE[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        Self {
            instance,
            adapter,
            device,
            queue,

            output,
        }
    }

    fn render(&self, pass: &PathTracingPass) {
        let mut cmd = self.device.create_command_encoder(&Default::default());

        let output = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pass.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &self.output.create_view(&Default::default()),
                ),
            }],
        });

        pass.execute(&self.device, &mut cmd, &output);

        self.queue.submit([cmd.finish()]);

        self.device.poll(wgpu::Maintain::Wait);
    }

    fn get_output_image(&self) -> Vec<u8> {
        let mut cmd = self.device.create_command_encoder(&Default::default());

        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (4 * OUTPUT_SIZE[0] * OUTPUT_SIZE[1]) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let copy_buf = wgpu::ImageCopyBuffer {
            buffer: &buf,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(OUTPUT_SIZE[0] * 4),
                rows_per_image: None,
            },
        };

        cmd.copy_texture_to_buffer(
            self.output.as_image_copy(),
            copy_buf,
            wgpu::Extent3d {
                width: OUTPUT_SIZE[0],
                height: OUTPUT_SIZE[1],
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit([cmd.finish()]);

        buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});

        self.device.poll(wgpu::Maintain::Wait);

        std::thread::sleep(Duration::from_secs(1));

        let view = buf.slice(..).get_mapped_range().to_owned();

        assert_eq!(view.len(), (4 * OUTPUT_SIZE[0] * OUTPUT_SIZE[1]) as usize);

        view
    }
}

fn save_png<P: AsRef<Path>>(path: P, data: Vec<u8>) -> Result<()> {
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, OUTPUT_SIZE[0], OUTPUT_SIZE[1]);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;

    Ok(())
}

pub struct PathTracingPass {
    bind_group_layout: wgpu::BindGroupLayout,
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

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    access: wgpu::StorageTextureAccess::WriteOnly,
                },
                count: None,
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &module,
            entry_point: ShaderStage::Compute.entry_point(),
        });

        Self {
            bind_group_layout,
            layout,
            pipeline,
        }
    }

    fn execute(
        &self,
        device: &wgpu::Device,
        cmd: &mut wgpu::CommandEncoder,
        output: &wgpu::BindGroup,
    ) {
        let mut pass = cmd.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

        pass.set_bind_group(0, output, &[]);
        pass.set_pipeline(&self.pipeline);

        let x_groups = OUTPUT_SIZE[0] / WORK_GROUP_SIZE[0];
        let y_groups = OUTPUT_SIZE[1] / WORK_GROUP_SIZE[1];

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

    let image = renderer.get_output_image();

    save_png("output.png", image);

    println!("Hello, world!");
}
