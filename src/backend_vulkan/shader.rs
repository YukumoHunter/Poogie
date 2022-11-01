use super::device::Device;
use anyhow::Result;
use ash::vk;
use naga::{
    back::spv::{self, PipelineOptions},
    front::glsl,
    front::wgsl,
    valid::{Capabilities, ValidationFlags},
};
use std::{fs, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderLanguage {
    GLSL,
    WGSL,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Clone, Debug)]
pub struct ShaderSourceBuilder {
    pub entry: String,
}

impl ShaderSourceBuilder {
    pub fn default() -> Self {
        ShaderSourceBuilder {
            entry: String::from("main"),
        }
    }

    pub fn entry(mut self, entry: String) -> Self {
        self.entry = entry;
        self
    }

    pub fn build(
        self,
        stage: ShaderStage,
        language: ShaderLanguage,
        path: PathBuf,
    ) -> ShaderSource {
        ShaderSource {
            stage,
            language,
            path,
            entry: self.entry,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShaderSource {
    pub stage: ShaderStage,
    pub language: ShaderLanguage,
    pub path: PathBuf,
    pub entry: String,
}

impl ShaderSource {
    pub fn builder() -> ShaderSourceBuilder {
        ShaderSourceBuilder::default()
    }

    pub fn create_shader(self) -> Result<Shader> {
        let source = self.clone();

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

        let pipeline_opts = &PipelineOptions {
            shader_stage: naga_stage,
            entry_point: module.entry_points[0].name.clone(),
        };

        let code = spv::write_vec(
            &module,
            &module_info,
            &spv::Options::default(),
            if self.language == ShaderLanguage::WGSL {
                None
            } else {
                Some(pipeline_opts)
            },
        )?;

        let shader = Shader { code, source };
        log::debug!("Loaded shader {:?}", shader.source);

        Ok(shader)
    }
}

pub struct Shader {
    pub code: Vec<u32>,
    pub source: ShaderSource,
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
