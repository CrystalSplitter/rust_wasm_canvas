import { webglMain } from "./shader_comp.js";
import "./index.scss";

const CANVAS = document.getElementById("drawscape");
//const FRAG_SHADER = "src/glsl_shaders/fragment_shader.glsl";
const FRAG_SHADER = "src/glsl_shaders/colourized_fs.glsl";
//const VERT_SHADER = "src/glsl_shaders/vertex_shader.glsl";
const VERT_SHADER = "src/glsl_shaders/ortho_3d_vs.glsl";

async function main() {
    const fragShaderSrcP = fetch(FRAG_SHADER).then(r => r.text());
    const vertShaderSrcP = fetch(VERT_SHADER).then(r => r.text());
    const fragShaderSrc = await fragShaderSrcP;
    const vertShaderSrc = await vertShaderSrcP;

    webglMain(CANVAS, fragShaderSrc, vertShaderSrc);
}

main();
