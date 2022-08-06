#[macro_use]
extern crate glium;

use std::time::Instant;
mod support;

use glium::{glutin, texture::UnsignedTexture2d, uniform, Surface, Texture2d};

fn main() {
    // building the display, ie. the main object
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_gl(glutin::GlRequest::Latest);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let start_time = Instant::now();

    let render_texture = UnsignedTexture2d::empty_with_format(
        &display,
        glium::texture::UncompressedUintFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        1024,
        1024,
    )
    .unwrap();

    let final_texture = Texture2d::empty_with_format(
        &display,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        1024,
        1024,
    )
    .unwrap();

    let pt_shader = glium::program::ComputeShader::from_source(&display, r#"\
#version 430
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
uniform uint uWidth;
uniform uint uHeight;
uniform float uTime;
uniform layout(binding=3, rgba8ui) writeonly uimage2D uSourceTexture;
void main() {
  ivec2 i = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec2 uv = vec2(i) * vec2(1.0 / float(uWidth), 1.0 / float(uHeight));

  // perform path tracing
  
  const int M =128;
  for (int i = 0; i<M; i++) { }

  vec4 color = vec4(1, 0, 0, 1);

  imageStore(uSourceTexture, i , uvec4(color * 255.0f));
}
    "#).unwrap();

    let fractal_shader = glium::program::ComputeShader::from_source(&display, r#"\
#version 430
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
uniform uint uWidth;
uniform uint uHeight;
uniform float uTime;
uniform layout(binding=3, rgba8ui) writeonly uimage2D uSourceTexture;
void main() {
  ivec2 i = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec2 uv = vec2(i) * vec2(1.0 / float(uWidth), 1.0 / float(uHeight));
  
  float n = 0.0;
  vec2 c = vec2(-.745, .186) +  (uv - 0.5)*(2.0+ 1.7*cos(1.8)  ), 
    z = vec2(0.0);
  const int M =128;
  for (int i = 0; i<M; i++)
    {
      z = vec2(z.x*z.x - z.y*z.y, 2.*z.x*z.y) + c;
      if (dot(z, z) > 2) break;
      n++;
    }
  vec3 bla = vec3(0,0,0.0);
  vec3 blu = vec3(0,0,0.8);
  vec4 color;
  if( n >= 0 && n <= M/2-1 ) { color = vec4( mix( vec3(0.2, 0.1, 0.4), blu, n / float(M/2-1) ), 1.0) ;  }
  if( n >= M/2 && n <= M ) { color = vec4( mix( blu, bla, float(n - M/2 ) / float(M/2) ), 1.0) ;  }
  imageStore(uSourceTexture, i , uvec4(color * 255.0f));
}
    "#).unwrap();

    let copy_shader = glium::program::ComputeShader::from_source(
        &display,
        r#"\
#version 430
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
uniform layout(binding=3, rgba8ui) readonly uimage2D uSourceTexture;
uniform layout(binding=4, rgba8) writeonly image2D destTexture;
void main() {
  ivec2 i = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec3 c = vec3(imageLoad(uSourceTexture, i).xyz);
  vec3 cnorm = c/255.0;
  imageStore(destTexture, i, vec4(cnorm,1.0));
}
    "#,
    )
    .unwrap();

    support::start_loop(event_loop, move |events| {
        let image_unit = render_texture
            .image_unit(glium::uniforms::ImageUnitFormat::RGBA8UI)
            .unwrap()
            .set_access(glium::uniforms::ImageUnitAccess::Write);

        pt_shader.execute(
            uniform! {
                uWidth: render_texture.width(),
                uHeight: render_texture.height(),
                uSourceTexture: image_unit,
                uTime: Instant::now().duration_since(start_time.clone()).as_secs_f32(),
            },
            render_texture.width(),
            render_texture.height(),
            1,
        );

        let source_unit = render_texture
            .image_unit(glium::uniforms::ImageUnitFormat::RGBA8UI)
            .unwrap()
            .set_access(glium::uniforms::ImageUnitAccess::Read);
        let final_unit = final_texture
            .image_unit(glium::uniforms::ImageUnitFormat::RGBA8)
            .unwrap()
            .set_access(glium::uniforms::ImageUnitAccess::Write);

        copy_shader.execute(
            uniform! {
                uSourceTexture: source_unit,
                destTexture: final_unit,
            },
            render_texture.width(),
            render_texture.height(),
            1,
        );

        // drawing a frame
        let target = display.draw();
        final_texture
            .as_surface()
            .fill(&target, glium::uniforms::MagnifySamplerFilter::Nearest);
        target.finish().unwrap();

        // polling and handling the events received by the window
        let mut action = support::Action::Continue;
        for event in events {
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => action = support::Action::Stop,
                    _ => (),
                },
                _ => (),
            }
        }

        action
    });
}