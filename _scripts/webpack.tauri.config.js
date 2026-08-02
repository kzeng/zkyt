const webpack = require('webpack')

const config = require('./webpack.renderer.config')

config.name = 'tauri-renderer'

config.plugins = config.plugins.map(plugin => {
  if (plugin instanceof webpack.DefinePlugin) {
    return new webpack.DefinePlugin({
      ...plugin.definitions,
      'process.env.IS_ELECTRON': false,
      'process.env.IS_TAURI': true,
      'process.env.SUPPORTS_LOCAL_API': false,
    })
  }

  return plugin
})

module.exports = config
