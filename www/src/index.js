import * as wasm from "wasm-canvas-js";
import { webglMain } from "./shader_comp.js";
import "./index.scss";

const CANVAS = document.getElementById("drawscape");

async function main() {
    const fragShaderSrcP = fetch("src/glsl_shaders/fragment_shader.glsl").then(r => r.text());
    const vertShaderSrcP = fetch("src/glsl_shaders/vertex_shader.glsl").then(r => r.text());
    const fragShaderSrc = await fragShaderSrcP;
    const vertShaderSrc = await vertShaderSrcP;

    webglMain(CANVAS, fragShaderSrc, vertShaderSrc);
}

main();
