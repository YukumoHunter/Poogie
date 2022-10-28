use super::device::Device;
use anyhow::Result;
use ash::vk;
use naga::{
    back::spv::{self, PipelineOptions},
    front::glsl,
    front::wgsl,
    valid::{Capabilities, ValidationFlags},
};
use std::{ffi::CString, fs, path::PathBuf};

#[derive(Clone, Copy, Debug)]
pub enum ShaderLanguage {
    GLSL,
    WGSL,
}

#[derive(Clone, Copy, Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Clone, Debug)]
pub struct ShaderDescBuilder {
    pub entry: CString,
}

impl ShaderDescBuilder {
    pub fn default() -> Self {
        ShaderDescBuilder {
            entry: CString::new("main").unwrap(),
        }
    }

    pub fn entry(mut self, entry: CString) -> Self {
        self.entry = entry;
        self
    }

    pub fn build(self, stage: ShaderStage, language: ShaderLanguage, path: PathBuf) -> ShaderDesc {
        ShaderDesc {
            stage,
            language,
            path,
            entry: self.entry,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShaderDesc {
    pub stage: ShaderStage,
    pub language: ShaderLanguage,
    pub path: PathBuf,
    pub entry: CString,
}

impl ShaderDesc {
    pub fn builder() -> ShaderDescBuilder {
        ShaderDescBuilder::default()
    }

    pub fn create_shader(self) -> Result<Shader> {
        let desc = self.clone();

        let buf = fs::read_to_string(self.path)?;

        let naga_stage = match self.stage {
            ShaderStage::Vertex => naga::ShaderStage::Vertex,
            ShaderStage::Fragment => naga::ShaderStage::Fragment,
            ShaderStage::Compute => naga::ShaderStage::Compute,
        };

        let module = match self.language {
            ShaderLanguage::GLSL => {
                let mut parser = glsl::Parser::default();
                let options = glsl::Options::from(naga_stage);
                parser
                    .parse(&options, &buf)
                    .expect("Failed to parse GLSL shader")
            }
            ShaderLanguage::WGSL => {
                let mut parser = wgsl::Parser::new();
                parser.parse(&buf).expect("Failed to parse WGSL shader")
            }
        };

        let module_info =
            naga::valid::Validator::new(ValidationFlags::empty(), Capabilities::empty())
                .validate(&module)?;

        let code = spv::write_vec(
            &module,
            &module_info,
            &spv::Options::default(),
            Some(&PipelineOptions {
                shader_stage: naga_stage,
                entry_point: module.entry_points[0].name.clone(),
            }),
        )?;

        let shader = Shader { code, desc };
        log::info!("Loaded shader {:?}", shader.desc);

        Ok(shader)
    }
}

pub struct Shader {
    pub code: Vec<u32>,
    pub desc: ShaderDesc,
}

impl Shader {
    pub fn create_module(&self, device: &Device) -> Result<vk::ShaderModule> {
        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(&self.code);

        Ok(unsafe {
            device
                .raw
                .create_shader_module(&shader_module_create_info, None)
                .expect("Error creating shader module")
        })
    }
}
