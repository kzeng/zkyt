process.env.NODE_ENV = 'development'

const path = require('path')
const webpack = require('webpack')
const WebpackDevServer = require('webpack-dev-server')

const ProcessLocalesPlugin = require('./ProcessLocalesPlugin')
const tauriConfig = require('./webpack.tauri.config')

const port = 9080
const SHAKA_LOCALES_TO_BE_BUNDLED = tauriConfig.SHAKA_LOCALES_TO_BE_BUNDLED
delete tauriConfig.SHAKA_LOCALES_TO_BE_BUNDLED

/**
 * @param {import('webpack').Compiler} compiler
 * @param {WebpackDevServer} devServer
 */
function setupNotifyLocaleUpdate(compiler, devServer) {
  const notifyLocaleChange = (updatedLocales) => {
    devServer.sendMessage(devServer.webSocketServer.clients, 'freetube-locale-update', updatedLocales)
  }

  compiler.options.plugins
    .filter(plugin => plugin instanceof ProcessLocalesPlugin)
    .forEach((/** @type {ProcessLocalesPlugin} */plugin) => {
      plugin.notifyLocaleChange = notifyLocaleChange
    })
}

function startTauriRenderer() {
  const compiler = webpack(tauriConfig)
  const { name } = compiler

  const server = new WebpackDevServer({
    client: {
      overlay: {
        runtimeErrors: false
      }
    },
    static: [
      {
        directory: path.resolve(__dirname, '..', 'static'),
        watch: {
          ignored: [
            /(dashFiles|storyboards)\/*/,
            '**/.DS_Store',
            '**/static/locales/*'
          ]
        },
        publicPath: '/static'
      },
      {
        directory: path.resolve(__dirname, '..', 'node_modules', 'shaka-player', 'ui', 'locales'),
        publicPath: '/static/shaka-player-locales',
        watch: {
          ignored: `**/!(${SHAKA_LOCALES_TO_BE_BUNDLED.join('|')}).json`
        }
      }
    ],
    port
  })

  server.apply(compiler)

  setupNotifyLocaleUpdate(compiler, server)

  compiler.watch({ aggregateTimeout: 250 }, (err, result) => {
    if (err) console.error(err)

    if (result) {
      console.log('\n' + result.toString({ colors: true }))
    }

    console.log(`\nCompiled ${name} script!\n\nWatching file changes for ${name} script...`)
  })
}

startTauriRenderer()
