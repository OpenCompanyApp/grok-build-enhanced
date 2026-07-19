# Third-Party Notices

The Kimi Code provider under `src/provider/kimi_code/` contains modified Rust
adaptations of public provider behavior from the
[Moonshot AI Kimi Code CLI](https://github.com/MoonshotAI/kimi-code), notably
the Chat Completions request contract, Kimi JSON Schema compatibility repair,
64-character tool-call-ID policy, proprietary streaming-usage placement, and
Kimi-over-Anthropic Messages behavior.

The implementation has been translated to Rust and substantially modified for
Grok Build's existing sampler types, provider-scoped authentication, bounded
request processing, no-redirect transport, cache-affinity privacy, and redacted
diagnostics. It is not a verbatim copy and does not send the official Kimi
client's identity headers.

MIT License

Copyright (c) 2026 Moonshot AI

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
