pub mod machine;
pub mod object;
pub mod resources;
pub mod camera;

use nalgebra as na;
use glium::{self, uniform};
use num_traits::ToPrimitive;

pub use object::{Object, Instance, InstanceParams};
pub use camera::Camera;
pub use resources::Resources;

use object::ObjectBuffers;

pub struct Context {
    pub camera: camera::Camera,
    pub elapsed_time_secs: f32,
}

#[derive(Default)]
pub struct RenderList {
    instances: Vec<Instance>,
}

impl RenderList {
    pub fn new() -> RenderList {
        Default::default()
    }

    pub fn add_instance(&mut self, instance: &Instance) {
        self.instances.push(instance.clone());
    }

    pub fn add(&mut self, object: Object, params: &InstanceParams) {
        self.add_instance(&Instance { object, params: params.clone() });
    }

    pub fn render<S: glium::Surface>(
        &self,
        resources: &Resources,
        context: &Context,
        params: &glium::DrawParameters,
        target: &mut S,
    ) -> Result<(), glium::DrawError> {
        // TODO: Could sort by object here to reduce state switching for large
        // numbers of objects.

        let mat_projection: [[f32; 4]; 4] = context.camera.projection().into();
        let mat_view: [[f32; 4]; 4] = context.camera.view().into();

        let params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. params.clone()
        };

        //let params = Default::default();

        for instance in &self.instances {
            let buffers = resources.get_object_buffers(instance.object);

            let mat_model: [[f32; 4]; 4] = instance.params.transform.into();
            let color: [f32; 4] = instance.params.color.into();
            let uniforms = uniform! {
                mat_model: mat_model, 
                mat_view: mat_view,
                mat_projection: mat_projection,
                color: color,
                t: context.elapsed_time_secs,
            };

            match &buffers.index_buffer {
                object::IndexBuffer::IndexBuffer(buffer) => {
                    target.draw(
                        &buffers.vertex_buffer,
                        buffer,
                        &resources.program,
                        &uniforms,
                        &params,
                    )?;
                }
                object::IndexBuffer::NoIndices(buffer) => {
                    target.draw(
                        &buffers.vertex_buffer,
                        buffer,
                        &resources.program,
                        &uniforms,
                        &params,
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }
}
