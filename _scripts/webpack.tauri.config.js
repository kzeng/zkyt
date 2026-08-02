const webpack = require('webpack')
const path = require('path')
const CopyWebpackPlugin = require('copy-webpack-plugin')

const config = require('./webpack.renderer.config')

config.name = 'tauri-renderer'

config.plugins = config.plugins.map(plugin => {
  if (plugin.constructor.name === 'ProcessLocalesPlugin') {
    plugin.compress = false
  }

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

config.plugins.push(new CopyWebpackPlugin({
  patterns: [
    {
      from: path.join(__dirname, '../static/invidious-instances.json'),
      to: 'static/invidious-instances.json'
    },
    {
      from: path.join(__dirname, '../static/geolocations/*.json').replaceAll('\\', '/'),
      to: 'static/geolocations/[name][ext]'
    }
  ]
}))

module.exports = config
