import { BotGuardClient } from 'bgutils-js/botguard'
import { buildURL, GOOG_API_KEY } from 'bgutils-js/utils'
import { WebPoMinter } from 'bgutils-js/webpo'

// This script has it's own webpack config, as it gets passed as a string to Electron's evaluateJavaScript function
// in src/main/poTokenGenerator.js

/**
 * Based on: https://github.com/LuanRT/BgUtils/blob/main/examples/node/innertube-challenge-fetcher-example.ts
 * @param {string} videoId
 * @param {import('youtubei.js').Session['context']} context
 */
export default async function (videoId, context, fetchFunc = fetch) {
  const requestKey = 'O43z0dpjhgX20SCx4KAo'

  const challengeResponse = await fetchFunc(
    'https://www.youtube.com/youtubei/v1/att/get?prettyPrint=false&alt=json',
    {
      method: 'POST',
      headers: {
        Accept: '*/*',
        'Content-Type': 'application/json',
        'X-Goog-Visitor-Id': context.client.visitorData,
        'X-Youtube-Client-Version': context.client.clientVersion,
        'X-Youtube-Client-Name': '1'
      },
      body: JSON.stringify({
        engagementType: 'ENGAGEMENT_TYPE_UNBOUND',
        context
      }),
    }
  )

  if (!challengeResponse.ok) {
    throw new Error(`Request to ${challengeResponse.url} failed with status ${challengeResponse.status}\n${await challengeResponse.text()}`)
  }

  const challengeData = await challengeResponse.json()

  if (!challengeData.bgChallenge) {
    throw new Error('Failed to get BotGuard challenge')
  }

  let interpreterUrl = challengeData.bgChallenge.interpreterUrl.privateDoNotAccessOrElseTrustedResourceUrlWrappedValue

  if (interpreterUrl.startsWith('//')) {
    interpreterUrl = `https:${interpreterUrl}`
  }

  const bgScriptResponse = await fetchFunc(interpreterUrl)
  const interpreterJavascript = await bgScriptResponse.text()

  if (interpreterJavascript) {
    // eslint-disable-next-line no-new-func
    new Function(interpreterJavascript)()
  } else {
    throw new Error('Could not load VM.')
  }

  const botGuard = await BotGuardClient.create({
    program: challengeData.bgChallenge.program,
    globalName: challengeData.bgChallenge.globalName,
    globalObject: window,
    userInteractionElement: document.documentElement
  })

  const webPoSignalOutput = []
  let botGuardResponse = await botGuard.snapshot({ webPoSignalOutput }, 10_000)

  if (webPoSignalOutput[0] === undefined) {
    try {
      botGuardResponse = await botGuard.snapshotSynchronous({ webPoSignalOutput })
    } catch (error) {
      console.warn('BotGuard synchronous snapshot failed', error)
    }
  }

  const integrityTokenResponse = await fetchFunc(buildURL('GenerateIT', true), {
    method: 'POST',
    headers: {
      Accept: '*/*',
      'content-type': 'application/json+protobuf',
      'x-goog-api-key': GOOG_API_KEY,
      'x-user-agent': 'grpc-web-javascript/0.1',
      Origin: 'https://www.youtube.com',
      Referer: 'https://www.youtube.com/',
      'Sec-Fetch-Mode': 'cors',
      'Sec-Fetch-Site': 'cross-site',
    },
    body: JSON.stringify([requestKey, botGuardResponse])
  })

  const integrityTokenResponseText = await integrityTokenResponse.text()
  let response

  try {
    response = JSON.parse(integrityTokenResponseText)
  } catch {
    throw new Error(
      `Could not parse integrity token response: status ${integrityTokenResponse.status}\n` +
      integrityTokenResponseText.slice(0, 500)
    )
  }

  const integrityToken = Array.isArray(response) && typeof response[0] === 'string'
    ? response[0]
    : null
  const websafeFallbackToken = Array.isArray(response) && typeof response[3] === 'string'
    ? response[3]
    : null

  if (integrityToken === null && websafeFallbackToken === null) {
    throw new Error(
      `Could not get integrity token: status ${integrityTokenResponse.status}\n` +
      integrityTokenResponseText.slice(0, 500)
    )
  }

  if (integrityToken === null) {
    console.warn('BotGuard returned only a websafe fallback token; skipping player poToken')
    return undefined
  }

  if (webPoSignalOutput[0] === undefined) {
    console.warn('BotGuard WebPO minter unavailable; skipping player poToken')
    return undefined
  }

  const integrityTokenBasedMinter = await WebPoMinter.create({ integrityToken }, webPoSignalOutput)

  return await integrityTokenBasedMinter.mintAsWebsafeString(videoId)
}
