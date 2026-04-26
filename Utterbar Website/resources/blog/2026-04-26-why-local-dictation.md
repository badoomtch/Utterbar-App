---
title: Why Utterbar runs on your PC, not in the cloud
excerpt: Local-only dictation isn't a privacy slogan — it's an architectural choice with real trade-offs and real benefits. Here's what they are.
---

Most modern dictation apps run in the cloud. You speak; your audio is uploaded to a server somewhere; transcribed text comes back. The architecture is everywhere because it's easy to ship, runs the latest models, and scales sideways with traffic.

Utterbar takes the other path. The transcription happens on your PC. The audio never leaves the machine. There's no account, no server-side anything, no internet required after the model is downloaded once.

This isn't an ideological choice. It's a practical one with real trade-offs. Worth being explicit about both sides.

## What you give up by running locally

**Bleeding-edge model freshness.** Cloud providers can swap a new model in overnight. Utterbar ships with NVIDIA's Parakeet ASR baked into the installer; you get a new model when you update the app, not the moment a paper drops.

**Cross-device sync.** Your vocabulary, modes, and history live on the PC where you typed them. There's no "log in on your laptop and your phone has the same setup". For a Windows-first dictation tool the trade isn't bad, but it is a trade.

**Lightweight devices.** Local ASR needs CPU or GPU. On a fanless ARM tablet, cloud transcription is faster because the cloud has more compute. Utterbar runs on Windows machines from roughly the last six years, but it isn't a phone-class workload.

## What you get back

**Latency that's actually low.** Audio doesn't take a round trip to a data centre. The first dictation in a session has a one-time cold start while the model loads; every dictation after that finishes in roughly the time it takes you to stop talking.

**Privacy that's verifiable, not promised.** Pull your ethernet cable. Open Process Hacker and look at Utterbar's network connections — there are none after the model finishes downloading. You don't have to trust a TOS that says "we don't store audio". The architecture means the question doesn't apply.

**No subscription tax on speaking.** Cloud ASR has a per-request cost — bandwidth, GPU minutes, model licensing. That cost gets passed on as a monthly fee, sometimes with word caps, often with a "fair use" clause that bites the heaviest users. A local model has no per-use cost. Utterbar charges £39.99 once, lifetime; the free tier is genuinely unlimited.

**Works with no internet.** On a train, on a plane, in a building with bad wifi, in a country with patchy infrastructure. The model is on the disk; the dictation works.

## When local doesn't make sense

If you're using dictation as a thin layer over a cloud LLM that does heavy rewriting — turning your spoken thoughts into polished prose, summarising meetings, writing code from voice — the LLM has to be in the cloud anyway. Local ASR plus cloud rewriting is one option. Cloud ASR with cloud rewriting in the same provider is simpler.

Utterbar isn't that. It's a tool for putting words you say into apps you use. The post-processing is deterministic and small (capitalisation, punctuation, simple replacements). If you want an AI ghost-writer, Utterbar isn't the answer. If you want fast accurate dictation that doesn't phone home, it is.

## The honest summary

Local-only is the right architecture for **a dictation utility**. It's the wrong architecture for **a writing assistant**. The Utterbar bet is that lots of people want the first thing and don't need the second.

If you've used cloud dictation tools and felt the lag, the word caps, the "you're offline" messages — that's exactly the experience this design avoids. [Download Utterbar for Windows](/download) — it's a one-minute install.
