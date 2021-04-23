import * as wasm from "wasm-canvas-js";
import { webglMain } from "./shader_comp.js"

const canvas = document.getElementById("drawscape");
const gl = canvas.getContext("webgl2");
//const ctx = canvas.getContext("2d");

class Rect {
    constructor(top, left, data) {
        this.top = top;
        this.left = left;
        this.data = data;
    }
}

function draw(ctx, new_data) {
    const imageWrapper = ctx.getImageData(0, 0, canvas.width, canvas.height);
    const data = imageWrapper.data;
    for (let i = 0; i < new_data.length; i++) {
        data[i] = new_data[i];
    }
    ctx.putImageData(imageWrapper, 0, 0);
}

async function main() {
    //const drawElem = document.getElementById("drawscape");
    /*
    const conf = wasm.UniverseConfig.new(2, 4, 3);
    const uni = wasm.Universe.new(1024, 768, conf);

    const updater = () => {
        let new_data = uni.to_image_data();
        draw(ctx, new_data);
        uni.update();
        setTimeout(updater, 1000/24);
    };
    updater();
    */
    //fetch("").then()
    const fragShaderSrcP = fetch("src/glsl_shaders/fragment_shader.glsl").then(r => r.text());
    const vertShaderSrcP = fetch("src/glsl_shaders/vertex_shader.glsl").then(r => r.text());
    const fragShaderSrc = await fragShaderSrcP;
    const vertShaderSrc = await vertShaderSrcP;
    webglMain(gl, fragShaderSrc, vertShaderSrc);
}

main();
