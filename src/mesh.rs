use crate::shader::ToUniforms;
pub use crate::CreationError;
use crate::{DrawError, Drawable, InstancingMode};

pub enum IndexBuffer {
    IndexBuffer(glium::index::IndexBuffer<u32>),
    NoIndices(glium::index::NoIndices),
}

impl<'a> From<&'a IndexBuffer> for glium::index::IndicesSource<'a> {
    fn from(buffer: &'a IndexBuffer) -> Self {
        match buffer {
            IndexBuffer::IndexBuffer(buffer) => buffer.into(),
            IndexBuffer::NoIndices(buffer) => buffer.into(),
        }
    }
}

pub struct Mesh<V: Copy> {
    pub vertex_buffer: glium::VertexBuffer<V>,
    pub index_buffer: IndexBuffer,
}

impl<V: glium::vertex::Vertex> Mesh<V> {
    pub fn create_with_indices<F: glium::backend::Facade>(
        facade: &F,
        primitive_type: glium::index::PrimitiveType,
        vertices: &[V],
        indices: &[u32],
    ) -> Result<Self, CreationError> {
        Ok(Mesh {
            vertex_buffer: glium::VertexBuffer::new(facade, vertices)?,
            index_buffer: IndexBuffer::IndexBuffer(glium::IndexBuffer::new(
                facade,
                primitive_type,
                indices,
            )?),
        })
    }
}

impl<V> Drawable<(), V> for Mesh<V>
where
    V: glium::vertex::Vertex,
{
    const INSTANCING_MODE: InstancingMode = InstancingMode::Uniforms;

    fn draw<U, S>(
        &self,
        program: &glium::Program,
        uniforms: &U,
        draw_params: &glium::DrawParameters,
        target: &mut S,
    ) -> Result<(), DrawError>
    where
        U: ToUniforms,
        S: glium::Surface,
    {
        let uniforms = uniforms.to_uniforms();

        target.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            program,
            &uniforms,
            draw_params,
        )?;

        Ok(())
    }
}
