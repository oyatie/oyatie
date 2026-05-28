---
doc_class: Onboarding
microservice: plugin-app-store
persona: marketplace-publisher
related_adrs: [ADR-0316, ADR-0249, ADR-0251]
date: 2026-05-20
doc_status: published
---

# plugin-app-store — Marketplace Publisher First Week

Audience: a developer/team publishing your first listing to the oyatie marketplace. You may be publishing a plugin, app, workflow, agent, model, or dataset (per ADR-0249's 6 categories).

## Day 1 — orientation + publisher account

Morning (3 h):
1. Receive `iam` invite. Cedar role `marketplace::publisher` binds: `marketplace::listing::{create,update,publish}`, `marketplace::analytics::read::own`, `marketplace::payment::receive`.
2. Log in to the publisher portal: `https://marketplace-publisher.<tenant>.oyatie.io`.
3. Configure your publisher profile: legal name, jurisdiction (for tax compliance), payment account (Stripe / Adyen), support contact, marketing URL, privacy policy URL, terms-of-service URL.
4. Sign the publisher agreement (provisioned via `contract-lifecycle-management` µservice). Without a signed PA, you cannot publish.

Afternoon (4 h):
5. Read the publisher primer (~ 45 min): portal → Help → "Publishing 101 + Manifest Reference".
6. Read the security + privacy guidelines (~ 30 min): portal → Help → "Security Requirements" + "Privacy Requirements".
7. Decide your listing category per ADR-0249 (plugins/apps/workflows/agents/models/datasets) and read the category-specific publish requirements.
8. Choose your initial pricing model: free, one-time purchase, subscription, usage-based, free + paid billing_components mix.

Deliverable: publisher profile complete + PA signed + category chosen + pricing model documented.

## Day 2 — manifest authoring

Morning (4 h):
1. Initialise your listing project: `oya marketplace init --category plugin --slug my-cool-plugin`. This scaffolds:
   - `oma.json` (manifest).
   - `README.md` (listing description).
   - `screenshots/` directory.
   - `LICENSE` file.
   - `SECURITY.md` (vulnerability disclosure policy).
   - `CHANGELOG.md`.

2. Edit `oma.json`:
   ```json
   {
     "schema_version": "1.0.0",
     "category": "plugin",
     "slug": "my-cool-plugin",
     "name": "My Cool Plugin",
     "version": "1.0.0",
     "description": "One-line description of what this plugin does",
     "long_description_markdown": "README.md",
     "publisher": "<your-publisher-id>",
     "license": "Apache-2.0",
     "homepage": "https://my-cool-plugin.example.com",
     "support_email": "support@your-publisher.com",
     "host_compatibility": {
       "oyatie_version": ">=2.5.0",
       "host_microservice": "docs"
     },
     "permissions_requested": [
       "docs::document::read",
       "docs::document::comment"
     ],
     "artifact": {
       "type": "container",
       "image": "registry.your-publisher.com/my-cool-plugin:1.0.0",
       "sbom": "sbom.cyclonedx.json"
     },
     "pricing": {
       "model": "free"
     },
     "categories": ["productivity", "collaboration"]
   }
   ```

Afternoon (3 h):
3. Validate the manifest: `oya marketplace validate ./oma.json`. Resolve schema errors.
4. Build + test your artifact locally per category-specific instructions.
5. Generate the SBOM: `oya marketplace sbom-generate --image registry.your-publisher.com/my-cool-plugin:1.0.0 --output sbom.cyclonedx.json`.

Deliverable: manifest validated + artifact built + SBOM emitted.

## Day 3 — security + license review

Morning (4 h):
1. Run the local security scan: `oya marketplace security-scan ./`. This invokes Trivy, Semgrep, ClamAV.
2. Review findings:
   - Critical / High CVEs: must fix or document mitigation.
   - Semgrep findings: review each; many are false-positives but security-sensitive ones (SQL injection, path traversal, secrets in code) MUST be fixed.
   - ClamAV: malware. Must be clean.
3. Re-run until scan is green (or has only acknowledged-mitigated findings).

Afternoon (3 h):
4. License compliance: `oya marketplace license-check ./`. Surfaces:
   - Your listing's declared license.
   - All transitive dependency licenses (from the SBOM).
   - Conflicts (e.g. you declared MIT but a dependency is GPL — copyleft virality).
5. Resolve conflicts: either remove the conflicting dependency, change your declared license, or document an explicit exception (rare).

Deliverable: clean security scan + license-compliant.

## Day 4 — listing assets + submission

Morning (4 h):
1. Add screenshots: 1280×800 PNG, 4-8 images showing key features. Substrate auto-resizes for marketplace display.
2. Author the README.md long description: clear value proposition, features list, getting-started snippet, screenshots inline.
3. Author SECURITY.md: vulnerability disclosure procedure, contact email, expected response time.
4. Author CHANGELOG.md: at least the initial 1.0.0 entry.

Afternoon (3 h):
5. Submit for review: `oya marketplace submit ./oma.json`. Substrate:
   - Re-validates manifest.
   - Re-runs security scans.
   - Re-checks license compliance.
   - Generates the SBOM canonical hash + signs it.
   - Routes to auto-review or manual review queue per the category + risk policy.
6. Track review status: portal → Listings → "Under review".

Deliverable: listing submitted.

## Day 5 — review feedback + publish

Morning (4 h):
1. Expect review feedback within 2-5 business days (manual) or within 90 s (auto-reviewable).
2. Common feedback:
   - Privacy policy missing or insufficient (e.g. doesn't disclose data collected).
   - Permissions over-requested ("Why do you need `docs::document::write` when you only read?").
   - Screenshot quality (blurry, doesn't show actual product).
   - Description vague or marketing-speak ("revolutionary AI-powered" without saying what it does).
3. Address feedback. Re-submit.

Afternoon (4 h):
4. Once approved, the listing publishes within 5 min.
5. Test the install flow: visit your listing's marketplace URL; install into a test tenant; verify the install completes + your plugin loads in the host µservice.
6. Configure analytics: portal → Listings → My Listing → "Analytics". Track installs, active users, ratings, payment revenue.
7. Author your launch announcement: blog post, social media, email to your contact list. The substrate provides referral links for tracking.

End of Week 1 deliverable: 1 listing published + 1 test install verified + launch comms drafted.

## What you should know by end of week 1

- Publisher account + agreement.
- `oma.json` manifest schema.
- Security scan (Trivy + Semgrep + ClamAV).
- License compliance (SPDX + dependency check).
- SBOM generation (CycloneDX).
- Listing submission + review workflow.
- Install flow + analytics.

## What you should NOT do in week 1

- Don't publish without an SBOM. Substrate refuses listings without one.
- Don't over-request permissions. Reviewers reject listings with broader permissions than the description justifies.
- Don't use copyleft (GPL/AGPL) dependencies in your listing if you're charging for it without complying with the license's source-availability obligations.
- Don't bypass the security scan with `--skip-security`. Substrate logs every bypass attempt to audit-chain.
- Don't submit pre-1.0 listings as v1.0.0. Use semver (`0.x.y` for beta).
