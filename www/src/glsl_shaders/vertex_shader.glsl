#version 300 es
// The line above this denotes we are using WebGL2.
 
// an attribute is an input (in) to a vertex shader.
// It will receive data from a buffer.
in vec2 a_position;

uniform vec2 u_resolution;
uniform mat3 u_transformationMatrix;

// all shaders have a main function
void main() {
  vec2 pixelSpacePosition = (u_transformationMatrix * vec3(a_position, 1.0)).xy;

  // Convert the position from pixels to 0.0 to 1.0
  vec2 zeroToOne = pixelSpacePosition / u_resolution;
  vec2 zeroToTwo = zeroToOne * 2.0;
  vec2 clipSpace = zeroToTwo - 1.0;

  // gl_Position is a special variable a vertex shader
  // is responsible for setting
  gl_Position = vec4(clipSpace, 0.0, 1.0);
}
