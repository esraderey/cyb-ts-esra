const path = require('path');
const rspack = require('@rspack/core');
// TODO: BootloaderPlugin uses deep webpack internals (compilation.addChunk, chunk.moveModule)
// that are not compatible with Rspack. Disabled until rewritten.
// const BootloaderPlugin = require('./src/components/loader/webpack-loader');

require('dotenv').config();

if (process.env.IPFS_DEPLOY) {
  // eslint-disable-next-line no-console
  console.log('*** IPFS Version ***');
}

const config = {
  devtool: 'cheap-module-source-map',
  entry: {
    main: [path.join(__dirname, 'src', 'index.tsx')],
  },
  output: {
    filename: '[name].js',
    path: path.join(__dirname, '/build'),
    publicPath: process.env.IPFS_DEPLOY ? './' : '/',
    assetModuleFilename: '[name].[hash:10][ext]',
    clean: true,
  },
  resolve: {
    fallback: {
      buffer: require.resolve('buffer'),
      fs: false,
      zlib: false,
      path: false,
      url: false,
      crypto: require.resolve('crypto-browserify'),
      assert: require.resolve('assert'),
      https: require.resolve('https-browserify'),
      os: require.resolve('os-browserify/browser'),
      http: require.resolve('stream-http'),
      stream: require.resolve('stream-browserify'),
      constants: require.resolve('constants-browserify'),
    },
    extensions: [
      '*',
      '.js',
      '.jsx',
      '.scss',
      '.svg',
      '.css',
      '.json',
      '.ts',
      '.tsx',
      '.mp3',
    ],
    alias: {
      'react/jsx-dev-runtime.js': 'react/jsx-dev-runtime',
      'react/jsx-runtime.js': 'react/jsx-runtime',
      src: path.resolve(__dirname, 'src/'),
      components: path.resolve(__dirname, 'src', 'components'),
      images: path.resolve(__dirname, 'src', 'image'),
      sounds: path.resolve(__dirname, 'src', 'sounds'),
    },
  },
  plugins: [
    new rspack.NormalModuleReplacementPlugin(/node:/, (resource) => {
      const mod = resource.request.replace(/^node:/, '');
      switch (mod) {
        case 'buffer':
          resource.request = 'buffer';
          break;
        case 'stream':
          resource.request = 'readable-stream';
          break;
        default:
          throw new Error(`Not found ${mod}`);
      }
    }),
    // BootloaderPlugin disabled — see TODO above
    new rspack.HtmlRspackPlugin({
      template: path.join(__dirname, 'src', 'index.html'),
      favicon: 'src/image/favicon.ico',
      filename: 'index.html',
      ...(process.env.IPFS_DEPLOY ? { publicPath: './' } : {}),
      inject: 'body',
      templateParameters: {
        COMMIT_SHA: process.env.COMMIT_SHA || '',
        BRANCH: process.env.BRANCH || '',
        NODE_ENV: process.env.NODE_ENV || 'development',
      },
    }),
    new rspack.CssExtractRspackPlugin({
      filename: '[name].css',
      chunkFilename: '[id].css',
    }),
    new rspack.DefinePlugin({
      'process.env.IPFS_DEPLOY': JSON.stringify(process.env.IPFS_DEPLOY),
      'process.env.COMMIT_SHA': JSON.stringify(process.env.COMMIT_SHA),
      'process.env.BRANCH': JSON.stringify(process.env.BRANCH),
      'process.env.CHAIN_ID': JSON.stringify(process.env.CHAIN_ID),
      'process.env.RPC_URL': JSON.stringify(process.env.RPC_URL),
      'process.env.LCD_URL': JSON.stringify(process.env.LCD_URL),
      'process.env.WEBSOCKET_URL': JSON.stringify(process.env.WEBSOCKET_URL),
      'process.env.INDEX_HTTPS': JSON.stringify(process.env.INDEX_HTTPS),
      'process.env.INDEX_WEBSOCKET': JSON.stringify(
        process.env.INDEX_WEBSOCKET
      ),
      'process.env.CYBER_GATEWAY': JSON.stringify(process.env.CYBER_GATEWAY),
      'process.env.BASE_DENOM': JSON.stringify(process.env.BASE_DENOM),
      'process.env.DENOM_LIQUID': JSON.stringify(process.env.DENOM_LIQUID),
      'process.env.BECH32_PREFIX': JSON.stringify(process.env.BECH32_PREFIX),
    }),
    new rspack.ProvidePlugin({
      cyblog: ['src/utils/logging/cyblog.ts', 'default'],
      process: 'process/browser',
      Buffer: ['buffer', 'Buffer'],
    }),
  ],
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
                },
              },
            },
            env: {
              targets: 'chrome >= 87, firefox >= 78, safari >= 14, edge >= 88',
            },
          },
        },
      },
      {
        include: /node_modules/,
        test: /\.mjs$/,
        type: 'javascript/auto',
      },
      {
        test: /\.css$/,
        use: [
          rspack.CssExtractRspackPlugin.loader,
          {
            loader: 'css-loader',
            options: {
              importLoaders: 2,
              sourceMap: false,
            },
          },
        ],
      },
      {
        test: /\.(scss|sass)$/,
        use: [
          'style-loader',
          {
            loader: 'css-loader',
            options: {
              importLoaders: 1,
              modules: {
                localIdentName: '[name]__[local]___[hash:base64:5]',
              },
            },
          },
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: [['postcss-media-minmax']],
              },
            },
          },
          'sass-loader',
        ],
      },
      {
        test: /\.(woff|woff2|eot|ttf|otf)$/,
        type: 'asset/resource',
      },
      {
        test: /\.(ogg|mp3|wav|mpe?g)$/i,
        type: 'asset/resource',
      },
      {
        test: /\.(jpe?g|png|gif|svg|ico)$/i,
        type: 'asset/resource',
      },
      {
        test: /\.m?js$/,
        resolve: {
          fullySpecified: false,
        },
      },
      {
        test: /\.cozo$/,
        type: 'asset/source',
      },
      {
        test: /\.(graphql|gql)$/,
        exclude: /node_modules/,
        use: 'graphql-tag/loader',
      },
      {
        test: /\.rn$/,
        type: 'asset/source',
      },
    ],
  },
};

module.exports = config;
