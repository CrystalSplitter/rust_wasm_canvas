#version 300 es
// The line above this denotes we are using WebGL2.
 
// an attribute is an input (in) to a vertex shader.
// It will receive data from a buffer.
in vec4 a_position;
 
// all shaders have a main function
void main() {
 
  // gl_Position is a special variable a vertex shader
  // is responsible for setting
  gl_Position = a_position;
}