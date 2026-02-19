const rspack = require('@rspack/core');
const { merge } = require('webpack-merge');
const ReactRefreshPlugin = require('@rspack/plugin-react-refresh');

const commonConfig = require('./rspack.config.common');

module.exports = merge(commonConfig, {
  mode: 'development',
  devServer: {
    server: 'https',
    host: 'localhost',
    port: process.env.PORT_APP || '3001',
    hot: true,
    // ngrok tunnel
    allowedHosts: ['.ngrok-free.app'],
    headers: {
      'Access-Control-Allow-Origin': '*',
    },
    historyApiFallback: true,
  },
  module: {
    rules: [
      {
        test: /\.[jt]sx?$/,
        exclude: /node_modules/,
        include: [/src/, /netlify\/mocks/],
        use: {
          loader: 'builtin:swc-loader',
          options: {
            jsc: {
              parser: {
                syntax: 'typescript',
                tsx: true,
              },
              transform: {
                react: {
                  runtime: 'automatic',
                  refresh: true,
                },
              },
            },
            env: {
              targets: 'chrome >= 87, firefox >= 78, safari >= 14, edge >= 88',
            },
          },
        },
      },
    ],
  },
  plugins: [
    new ReactRefreshPlugin({ overlay: false }),
    new rspack.DefinePlugin({
      ...commonConfig.plugins.find(
        (plugin) => plugin.constructor.name === 'DefinePlugin'
      ).definitions,
      'process.env.IS_DEV': JSON.stringify(true),
    }),
  ],
});
