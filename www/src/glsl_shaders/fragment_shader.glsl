#version 300 es
// Line above this indicates we're using WebGL2.
    
// fragment shaders don't have a default precision so we need
// to pick one. highp is a good default. It means "high precision"
precision highp float;
    
// we need to declare an output for the fragment shader
out vec4 outColor;

// Set color.
uniform vec4 u_color;

void main() {
    // Just set the output to a constant reddish-purple
    outColor = u_color;
}
