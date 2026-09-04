# Demo Delivery

## Outcome

Starting with the first runnable milestone, the latest successful ThorUI demo is served at `https://thorui.yougotserved.dev`. DNS hostnames are case-insensitive; project files use the lowercase canonical form.

The URL always points to the last green default-branch build. A broken build, failed smoke check, or interrupted deployment cannot replace it.

## Existing infrastructure fit

The sibling repository at `/home/kadajett/Dev/kadajett-infrastructure` is the Pulumi control plane for shared infrastructure and public Cloudflare bindings. It already separates application deployment from edge-domain ownership:

- application tooling deploys Worker code;
- an independent Pulumi edge project owns and protects Worker custom domains;
- tunnels exist for private origins, not for originless Worker applications.

ThorUI follows that split. It does not add a Kubernetes workload or Cloudflare Tunnel for a static WASM demo.

## Ownership

### ThorUI repository

This repository owns:

- the reproducible Rust/WASM, JavaScript glue, CSS, and asset build;
- a pinned Wrangler development dependency and configuration;
- the `thorui-demo` Worker service, Static Assets upload, and report storage binding;
- build metadata, offline behavior, cache policy, and smoke tests;
- the command used by local development and CI to deploy a candidate.

The Wrangler configuration must not declare the production custom domain. This prevents two tools from owning the same binding.

### Infrastructure repository

`kadajett-infrastructure` owns:

- the `thorui.yougotserved.dev` custom-domain binding;
- Cloudflare account and `yougotserved.dev` zone identifiers;
- resource protection, creation or adoption, and Pulumi state;
- the least-privilege credential boundary for domain management.

Implementation should add a small `thorui-edge` Pulumi project. Worker-domain behavior should be extracted from the current tunnel module into a reusable deep module instead of copying its validation, provider, protection, and import logic.

The infrastructure worktree inspection on 2026-09-04 found unrelated local changes. Future implementation must re-read it and preserve those changes before editing.

## Cloudflare shape

Use Cloudflare Workers Static Assets for the offline CSR build. Cloudflare now directs new projects toward Workers, and a Worker Custom Domain is the recommended origin for an application with no external server.

Static assets bypass the Worker script. Only `/api/*` invokes the script used for capability-report intake. Its configuration needs:

- a current, pinned compatibility date;
- the built asset directory;
- single-page-application fallback only if client routes are introduced;
- no production routes or custom-domain declaration;
- `workers_dev` retained for bootstrap and diagnostic access if policy permits.

Capability reports use a ThorUI-owned KV namespace. The public API accepts same-origin schema version 1 reports, caps their size, applies an edge rate limit, and expires them after 90 days. It exposes no public list or read route.

The first bootstrap is ordered:

1. Deploy `thorui-demo` to its Workers development hostname.
2. Smoke-test the root document, WASM, CSS, offline shell, and build metadata.
3. Apply the `thorui-edge` Pulumi stack to create the custom-domain binding.
4. Verify TLS, origin isolation, asset caching, and the visible build revision.

Later releases update only the Worker artifact. Pulumi changes only when the edge contract changes.

## Release contract

Every deployable build produces:

- content-hashed WASM, JavaScript, CSS, fonts, and media;
- a short-lived or revalidated HTML shell and service-worker entry;
- `version.json` with source revision, build time, schema versions, and channel;
- a machine-readable asset manifest for smoke checks;
- raw and compressed size reports for the performance budget.

Tag builds also produce a signed, R8-optimized Android APK and its SHA-256 checksum. The release key is held in GitHub Actions secrets and backed up outside the repository. The site publishes the certificate fingerprint for verified App Links, never the private key.

Hashed assets may be cached as immutable. The shell, version file, web manifest, and service-worker entry must revalidate so `latest` does not remain stale behind browser or CDN caches.

Service-worker updates are explicit. A running experience is not replaced mid-session; it reports that a newer build is ready and activates it on a safe reload. Saved-state and peer-protocol compatibility are checked before activation.

## Pipeline gate

The production promotion runs only after:

1. formatting, Clippy, native tests, and WASM checks pass;
2. the optimized static build is reproducible;
3. browser smoke tests pass at both Thor surface profiles;
4. offline reload and cache-update tests pass;
5. bundle and startup budgets pass;
6. the deployment identity matches the default branch and expected Worker;
7. the candidate is uploaded as a Worker version and passes the same checks at its version preview URL.

The accepted version is then promoted to production. An external smoke check verifies the canonical URL, TLS, build revision, critical assets, and security headers. A post-promotion failure rolls back to the recorded last-green version.

Preview builds use a non-production Worker URL or version preview. They never claim `thorui.yougotserved.dev`.

## Security boundary

- Deployment tokens live in CI or Pulumi secret configuration only.
- The artifact token can edit the ThorUI Worker but cannot edit zones.
- The infrastructure token can manage the required custom domain and zone data but does not build application code.
- No source maps containing local paths or source text ship by default.
- A restrictive Content Security Policy is introduced before third-party content or network APIs.

## Sources

- [Cloudflare recommends Workers for new projects](https://developers.cloudflare.com/pages/get-started/)
- [Workers Static Assets](https://developers.cloudflare.com/workers/static-assets/)
- [Worker Custom Domains](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/)
- [Static Assets configuration](https://developers.cloudflare.com/workers/static-assets/binding/)
- [Worker versions and deployments](https://developers.cloudflare.com/workers/versions-and-deployments/)
- [Version preview URLs](https://developers.cloudflare.com/workers/versions-and-deployments/preview-urls/)
