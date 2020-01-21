//! A first attempt to get particles into rendology.
//!
//! For now I'll just try to reimplement particle-frenzy:
//! https://github.com/leod/particle-frenzy
//!
//! I'm not sure if it makes sense to include particles in deferred shading,
//! but it seems overkill for now. Unfortunately, that would be somewhat
//! difficult as of now anyway, since geometry shaders are not part of the
//! design of `shader::Core`.

mod scene;

use crate::shader::InstanceInput;
use crate::error::{CreationError, DrawError};

pub use scene::{Params, Particle, Shader};

pub const PARTICLES_PER_BUFFER: usize = 10000;

/// Keeps a buffer of particle vertices.
struct Buffer {
    buffer: glium::VertexBuffer<<Particle as InstanceInput>::Vertex>,

    /// Time at which our most long-living particle dies.
    max_death_time: f32,
}

impl Buffer {
    fn create<F: glium::backend::Facade>(facade: &F, size: usize) -> Result<Self, CreationError> {
        let buffer = glium::VertexBuffer::empty_dynamic(facade, size)?;

        Ok(Self {
            buffer,
            max_death_time: 0.0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub particles_per_buffer: usize,
    pub num_buffers: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            particles_per_buffer: 10000,
            num_buffers: 10,
        }
    }
}

/// A particle system manages multiple buffers to store particles and allows
/// for rendering htem.
pub struct System {
    /// Our configuration.
    config: Config,

    /// A ring buffer of particles.
    buffers: Vec<Buffer>,

    /// The position at which the next particle will be inserted in our buffers.
    ///
    /// The first element is an index into `buffers`, the second element is an
    /// index into that buffer.
    ///
    /// `next_index` must always be valid.
    next_index: (usize, usize),
}

impl System {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
    ) -> Result<Self, CreationError> {
        assert!(config.particles_per_buffer > 0);
        assert!(config.num_buffers > 0);

        let buffers = (0..config.num_buffers)
            .map(|_| Buffer::create(facade, config.particles_per_buffer))
            .collect::<Result<Vec<_>, _>>()?;
 
        Ok(Self {
            config: config.clone(),
            buffers,
            next_index: (0, 0),
        })
    }

    pub fn shader(&self) -> Shader {
        Shader
    }

    pub fn spawn(&mut self, mut particles: &[<Particle as InstanceInput>::Vertex]) {
        // Copy new particles, filling up the ring buffer.
        while !particles.is_empty() {
            // By our invariant, `self.next_index` always is valid.
            assert!(self.next_index.0 < self.buffers.len());
            assert!(self.next_index.1 < self.config.particles_per_buffer);

            // Contiguously fill up the current buffer as much as possible.
            let capacity = self.config.particles_per_buffer - self.next_index.1;
            let num_to_write = particles.len().min(capacity);
            assert!(num_to_write > 0);

            let slice_to_write = &particles[0..num_to_write];
            let target_buffer = &mut self.buffers[self.next_index.0];

            target_buffer
                .buffer
                .slice_mut(self.next_index.1..self.next_index.1 + num_to_write)
                .unwrap() // safe to unwrap, since range bounded to capacity.
                .write(slice_to_write);

            // Keep track of how alive the buffer is. This allows us to ignore
            // buffers containing only dead particles when rendering.
            let new_max_death_time = slice_to_write
                .iter()
                .map(|particle| particle.spawn_time + particle.life_duration)
                .fold(0.0, f32::max);

            target_buffer.max_death_time = target_buffer.max_death_time.max(new_max_death_time);

            // Determine the next particle writing point.
            self.next_index = if num_to_write == capacity {
                // The current buffer is now fully written. Move on to the
                // next buffer in the ring.
                let buffer_index = (self.next_index.0 + 1) % self.buffers.len();
                (buffer_index, 0)
            } else {
                // Stay in the current buffer, advancing the inner index.
                (self.next_index.0, self.next_index.1 + num_to_write)
            };

            // Reduce the slice of particles to write for the next iteration.
            particles = &particles[num_to_write..];
        }
    }
}