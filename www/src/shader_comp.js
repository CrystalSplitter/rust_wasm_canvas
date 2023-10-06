import { bind_game, get_transform_mat } from "wasm-canvas-js";
import { InputBindings, setMousePos } from "./get_mouse";

/**
 *
 * @param {*} gl
 * @param {*} type Either gl.VERTEX_SHADER or gl.FRAGMENT_SHADER
 * @param {*} source Source code string.
 * @returns Created shader or null
 */
function createShader(gl, type, source) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  const success = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
  if (success) {
    return shader;
  } else {
    const infoLog = gl.getShaderInfoLog(shader);
    console.error("Failed to create WebGL2 Shader: ", infoLog);
    gl.deleteShader(shader);
    return null;
  }
}

/**
 *
 * @param {*} gl
 * @param {*} vertexShader
 * @param {*} fragmentShader
 * @returns {Program|null} Created program or null
 */
function createProgram(gl, vertexShader, fragmentShader) {
  const program = gl.createProgram();
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  gl.linkProgram(program);
  const success = gl.getProgramParameter(program, gl.LINK_STATUS);
  if (success) {
    return program;
  } else {
    const infoLog = gl.getProgramInfoLog(program);
    console.log("Failed to create WebGL2 Program: ", infoLog);
    gl.deleteProgram(program);
    return null;
  }
}

/**
 * @returns Promise Promise which contains an array of all the preloaded url contents.
 */
function preloads(urls) {
  let promises = [];
  names.forEach((n) => {
    promises.push(fetch(n));
  });
  return Promise.all(promises).then((responses) => {
    return Promise.all(responses.map(r => r.text()));
  });
}

export function webglMain(canvas, fragShaderSrc, vertShaderSrc) {
  const gl = canvas.getContext("webgl2", { antialias: false });
  /*
    // ================================================================
    // -- Set up buffers --
    // ================================================================
    // Bind the position buffer to the webgl ARRAY_BUFFER.
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    */

  // ================================================================
  // -- Program Creation --
  // ================================================================
  // Create the base shaders, and make the program.
  const vertShader = createShader(gl, gl.VERTEX_SHADER, vertShaderSrc);
  const fragShader = createShader(gl, gl.FRAGMENT_SHADER, fragShaderSrc);
  const prgm = createProgram(gl, vertShader, fragShader);
  gl.useProgram(prgm);

  /*
    // ================================================================
    // -- VAO --
    // ================================================================
    // Retrieve the attribute so we can use it later.
    const positionAttributeLocation = gl.getAttribLocation(prgm, "a_position");

    // Vertex Array Object
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);
    gl.enableVertexAttribArray(positionAttributeLocation);

    // Note that because the shader takes in a vec4, this will only set the x and y positions of the shader.
    const iterationSize = 3;
    const dataType = gl.FLOAT;
    const normalize = false;
    const stride = 0; // 0 means move forward based on the iteration Size.
    const offset = 0; // Start at the beginning of the buffer.
    gl.vertexAttribPointer(positionAttributeLocation, iterationSize, dataType, normalize, stride, offset);
    */

  // ================================================================
  // -- Canvas Set up --
  // ================================================================
  // Here is when we may want to resize the canvas...
  // Resize the viewport to the current canvas size.
  gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);

  // Set the clear color.
  gl.clearColor(0.8, 0.8, 1.0, 1.0);
  // Clear the color buffer.
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

  // ================================================================
  // -- Program variable set ups --
  // ================================================================
  let resolutionUniformLocation = gl.getUniformLocation(prgm, "u_resolution");
  gl.uniform2f(resolutionUniformLocation, gl.canvas.width, gl.canvas.height);

  let colorULoc = gl.getUniformLocation(prgm, "u_color");
  gl.uniform4f(colorULoc, 1.0, 0, 0, 1.0);

  // ================================================================
  // --  Draw using the program --
  // ================================================================

  preloads([""]).then(preloaded => {
    let preload_wrapper = { meshes: preloaded };
    bind_game(gl, prgm, ["u_transformationMatrix"], canvas, preload_wrapper);
  });

  //const drawScene = () => {
  //const transformMat = get_transform_mat(
  //    0, 0
  //inputBinding.mouseX/1.0,
  //-inputBinding.mouseY/1.0 + canvas.height,
  //);
  //gl.uniformMatrix3fv(transformationMatrixULoc, false, transformMat);
  //gl.drawArrays(primitiveType, arrayOffset, vertCount);
  //setTimeout(drawScene, 1000/60);
  //}
  //drawScene();
}
