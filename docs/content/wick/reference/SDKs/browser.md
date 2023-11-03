---
title: Browser Client
weight: 1
---

# Running components in a browser

Wick has a JavaScript client that can be used to run components in a browser. The client is available on [npm](https://www.npmjs.com/package/@candlecorp/wick), the repository is public on GitHub at [candlecorp/wick-js](https://github.com/candlecorp/wick-js), and the [component loader](/docs/component-loader) embedded below is a SvelteKit application you can clone at [candlecorp/wick-component-loader](https://github.com/candlecorp/wick-component-loader).

Use the embedded loader below to experiment with client-side WebAssembly components in your browser.

Drag and drop a WebAssembly component onto the loader below to run it in the browser.

_Note: This loader executes a WebAssembly file directly, bypassing the need to include a `.wick` file_


{{< rawhtml >}}
<iframe src="/component-loader" style="width:100%;height:400px;border:0;"></iframe>
{{< /rawhtml >}}
