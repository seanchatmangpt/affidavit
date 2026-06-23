
import os

raw_projects = """
/Users/sac/kgn
/Users/sac/mcpp
/Users/sac/clnrm
/Users/sac/gitvan
/Users/sac/yaml-server
/Users/sac/claude-desktop-context
/Users/sac/dteam
/Users/sac/wasm4pm
/Users/sac/clawdbot
/Users/sac/a2a-rs
/Users/sac/rocket-craft
/Users/sac/erlmcp
/Users/sac/nuxt-layer
/Users/sac/unrdf
/Users/sac/unibit
/Users/sac/stpnt
/Users/sac/yawlv6
/Users/sac/A2A
/Users/sac/cre
/Users/sac/chatmangpt
/Users/sac/clawd
/Users/sac/knowd
/Users/sac/knowtro
/Users/sac/zoeapp
/Users/sac/remo
/Users/sac/affidavit
/Users/sac/bytestar
/Users/sac/lsp-types-max
/Users/sac/wasm4pm-compat
/Users/sac/lsp-max
/Users/sac/chicago-tdd-tools
/Users/sac/teleport
/Users/sac/yawl
/Users/sac/process-intelligence
/Users/sac/clap-noun-verb
/Users/sac/capability-map
/Users/sac/autotel
/Users/sac/ggen
/Users/sac/open-ontologies
/Users/sac/bitstar
/Users/sac/obsr
/Users/sac/mcp-mqtt-erl
/Users/sac/ai
/Users/sac/optimus
/Users/sac/knhk
/Users/sac/zod-to-from
/Users/sac/un-test-utils
/Users/sac/kgc-sidecar
/Users/sac/compiled-cognition-hub
/Users/sac/unlsp
/Users/sac/ggen-mcp
/Users/sac/full-stack-rubric
/Users/sac/semantic_bit
/Users/sac/dogturk
/Users/sac/intvw
/Users/sac/universe-chain
/Users/sac/powlv2lsp
/Users/sac/insa
/Users/sac/neako
/Users/sac/practice
/Users/sac/citty-test-utils
/Users/sac/sos
/Users/sac/unjucks
/Users/sac/bcinr
/Users/sac/nehemiah-52
/Users/sac/tower-lsp-composition
/Users/sac/wf
/Users/sac/zoela
/Users/sac/doctester
/Users/sac/ggen-spec-kit
"""

projects = [p.strip() for p in raw_projects.split("\n") if p.strip()]

os.makedirs("nexus_reviews", exist_ok=True)

for proj_path in projects:
    proj_name = os.path.basename(proj_path)
    if proj_name == "affidavit":
        continue
    
    review_path = "nexus_reviews/" + proj_name + "_nexus_review.md"
    
    with open(review_path, "w") as f:
        f.write("# Nexus Integration Review: `" + proj_name + "`\n\n")
        f.write("## 1. Project Context\n")
        f.write("**Location:** `" + proj_path + "`\n")
        f.write("**Analysis Target:** `" + proj_name + "`\n\n")
        f.write("This document evaluates `" + proj_name + "` through the lens of the **Affidavit Nexus**. The goal is to determine how this external project can be integrated, upgraded, or deprecated in light of Combinatorial Maximalism, Cryptographic Provenance, and Bipartite Typestate Enforcement.\n\n")
        f.write("## 2. Structural Evaluation (The Chatman Equation)\n")
        f.write("Does `" + proj_name + "` adhere to the strict separation of Ontology and Manufacturing?\n")
        f.write("*   **Current State:** Likely operates on heuristic, ad-hoc programming paradigms.\n")
        f.write("*   **Nexus Upgrade Path:** The project core logic must be mapped to the `affi-cli.ttl` ontology. Its execution flow must be constrained by zero-cost Rust typestates, physically preventing invalid state transitions before compilation.\n\n")
        f.write("## 3. Cryptographic Provenance Integration\n")
        f.write("How can `" + proj_name + "` generate verifiable, append-only truth?\n")
        f.write("*   **Current State:** Likely relies on standard application logging (e.g., stdout, unstructured JSON), which is forgeable and structurally agnostic.\n")
        f.write("*   **Nexus Upgrade Path:** All state-mutating actions within `" + proj_name + "` must be wrapped in the `affidavit::emit!` macro. Every significant operation must yield a BLAKE3 cryptographic receipt. If the project interacts with other systems, it must pass the cryptographic seal to establish a verifiable chain of custody.\n\n")
        f.write("## 4. Process Mining & Conformance (wasm4pm)\n")
        f.write("*   **Current State:** Process execution is hidden within code paths. Deviations are only caught if a specific unit test fails.\n")
        f.write("*   **Nexus Upgrade Path:** By adopting the Affidavit event emission standard, `" + proj_name + "` automatically becomes compatible with the Heuristic Inductive Miner and alignment-based conformance checking. We can now mathematically prove whether its runtime behavior conforms to its design topology.\n\n")
        f.write("## 5. Verdict\n")
        f.write("**Status:** Requires Architectural Alignment.\n")
        f.write("**Action:** Deploy the Ostar Generative Pipeline to synthesize the boilerplate bindings between `" + proj_name + "` and the Affidavit core library. Treat `" + proj_name + "` as a sub-graph of the Universal Provenance Ontology.\n")

print("Generated " + str(len(projects)-1) + " nexus review files in ./nexus_reviews/")

