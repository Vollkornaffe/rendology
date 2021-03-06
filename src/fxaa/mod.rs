mod shaders;

use log::info;

use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerWrapFunction};
use glium::{uniform, Program, Surface, Texture2d};

use crate::{shader, DrawError, ScreenQuad};

pub use crate::CreationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Low,
    Medium,
    High,
}

impl Quality {
    pub fn exploration_offsets(&self) -> &[f32] {
        match self {
            Quality::Low => shaders::EXPLORATION_OFFSETS_LOW,
            Quality::Medium => shaders::EXPLORATION_OFFSETS_MEDIUM,
            Quality::High => shaders::EXPLORATION_OFFSETS_HIGH,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub quality: Quality,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            quality: Quality::Low,
        }
    }
}

pub struct FXAA {
    program: Program,
    screen_quad: ScreenQuad,
}

impl FXAA {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
    ) -> Result<Self, CreationError> {
        info!("Creating FXAA program");
        let core = shaders::postprocessing_core(config.quality.exploration_offsets());

        let program = core.build_program(facade, shader::InstancingMode::Uniforms)?;

        info!("Creating screen quad");
        let screen_quad = ScreenQuad::create(facade)?;

        Ok(FXAA {
            program,
            screen_quad,
        })
    }

    pub fn draw<S: Surface>(&self, texture: &Texture2d, target: &mut S) -> Result<(), DrawError> {
        let texture_map = Sampler::new(texture)
            .magnify_filter(MagnifySamplerFilter::Linear)
            .minify_filter(MinifySamplerFilter::Linear)
            .wrap_function(SamplerWrapFunction::Clamp);

        target.draw(
            &self.screen_quad.vertex_buffer,
            &self.screen_quad.index_buffer,
            &self.program,
            &uniform! {
                input_texture: texture_map,
            },
            &Default::default(),
        )?;

        Ok(())
    }
}
