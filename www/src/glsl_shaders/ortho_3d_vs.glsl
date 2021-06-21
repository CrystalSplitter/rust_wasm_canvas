#version 300 es
// The line above this denotes we are using WebGL2.
 
// an attribute is an input (in) to a vertex shader.
// It will receive data from a buffer.
in vec4 a_position;
in vec4 a_color;

uniform vec2 u_resolution;
uniform mat4 u_transformationMatrix;

out vec4 v_color;

// all shaders have a main function
void main() {
  gl_Position = u_transformationMatrix * a_position;

  // Pass colour directly to the fragment shader
  v_color = a_color;
}
