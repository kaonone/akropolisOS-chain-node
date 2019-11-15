/* eslint-disable import/no-extraneous-dependencies */
import path from 'path';
import { CleanWebpackPlugin } from 'clean-webpack-plugin';
import FileManagerWebpackPlugin from 'filemanager-webpack-plugin';
import webpack from 'webpack';
import HtmlWebpackPlugin from 'html-webpack-plugin';
import CircularDependencyPlugin from 'circular-dependency-plugin';
import FaviconsWebpackPlugin from 'favicons-webpack-plugin';

const forGhPages = true;
const pageTitle = 'Ethereum starter kit';

function sortChunks(a: webpack.compilation.Chunk, b: webpack.compilation.Chunk): number {
  const order = ['app', 'vendors', 'runtime'];
  return (
    order.findIndex(
      // webpack typings for Chunk are not correct wait for type updates for webpack.compilation.Chunk
      item => (b as any).names[0].includes(item), // eslint-disable-line @typescript-eslint/no-explicit-any
    ) - order.findIndex(item => (a as any).names[0].includes(item)) // eslint-disable-line @typescript-eslint/no-explicit-any
  );
}

const config: webpack.Configuration = {
  target: 'web',
  context: path.resolve(__dirname, '..', 'src'),
  entry: path.resolve(__dirname, '..', 'src', 'index.tsx'),
  output: {
    publicPath: '/',
    path: path.resolve(__dirname, '..', 'build'),
    filename: `js/[name]-[hash].bundle.js`,
    chunkFilename: `js/[name]-[hash].bundle.js`,
  },
  resolve: {
    modules: ['node_modules', 'src'],
    extensions: ['.js', '.jsx', '.ts', '.tsx', '.json'],
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: {
          loader: 'ts-loader',
          options: {
            logLevel: 'error',
          },
        },
      },
      {
        test: /\.(ttf|eot|woff(2)?)(\?[a-z0-9]+)?$/,
        use: 'file-loader?name=fonts/[hash].[ext]',
      },
      {
        test: /\.(png|svg)/,
        loader: 'url-loader',
        options: {
          name: 'images/[name].[ext]',
          limit: 10000,
        },
      },
    ],
  },
  plugins: [
    new CleanWebpackPlugin({
      cleanOnceBeforeBuildPatterns: ['build'],
    }),
    new HtmlWebpackPlugin({
      filename: 'index.html',
      template: 'core/index.html',
      chunksSortMode: sortChunks,
      title: pageTitle,
    }),
    new FaviconsWebpackPlugin(path.resolve(__dirname, '..', 'src', 'assets', 'favicon.png')),
    // new webpack.ContextReplacementPlugin(/moment[/\\]locale$/, new RegExp(LANGUAGES.join('|'))),
    new CircularDependencyPlugin({
      exclude: /node_modules/,
      failOnError: true,
    }),
    new webpack.HotModuleReplacementPlugin(),
  ].concat(
    forGhPages
      ? [
          // http://www.backalleycoder.com/2016/05/13/sghpa-the-single-page-app-hack-for-github-pages/
          new HtmlWebpackPlugin({
            filename: '404.html',
            template: 'core/index.html',
            chunksSortMode: sortChunks,
            title: pageTitle,
          }),
          new FileManagerWebpackPlugin({
            onEnd: {
              copy: [
                {
                  source: `assets/ghPageRoot/**`,
                  destination: `build`,
                },
              ],
            },
          }),
        ]
      : [],
  ),
  optimization: {
    runtimeChunk: 'single',
    splitChunks: {
      chunks: 'all',
    },
  },
  stats: {
    // typescript would remove the interfaces but also remove the imports of typings
    // and because of this, warnings are shown https://github.com/TypeStrong/ts-loader/issues/653
    warningsFilter: /export .* was not found in/,
    assets: false,
    modules: false,
  },
};

// eslint-disable-next-line import/no-default-export
export default config;
