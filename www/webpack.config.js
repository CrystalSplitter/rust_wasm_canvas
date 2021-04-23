const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./src/bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  experiments: {
    syncWebAssembly: true,
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
          {from: 'src/index.html', to: './'},
          {from: 'src/bootstrap.js', to: './'},
          {from: 'src/index.js', to: './'},
          {from: 'src/glsl_shaders/*'}
      ],
    }),
  ],
};
