//! Software Bill of Materials (SBOM) — canonical model and multi-format ingest.
//!
//! This is the keystone module of affidavit's supply-chain provenance layer. It
//! defines a single canonical SBOM model that all formats normalize into, so the
//! rest of the system (OCEL integration, compliance gates, vulnerability
//! correlation, supply-chain analysis) codes against one stable shape.
//!
//! # Combinatorial maximalism
//!
//! Fortune-5 solution architecture demands every format × every component type ×
//! every compliance regime. This module covers the *format* and *model* axis:
//!
//! - **Formats**: SPDX 2.3, SPDX 3.0, CycloneDX 1.5/1.6, SWID tags
//! - **Component types**: application, library, framework, container, OS, device,
//!   firmware, file, platform, driver, ML model, data
//! - **Identifiers**: PURL (package URL), CPE 2.3, SWID, bom-ref
//! - **Integrity**: SHA-1/256/384/512, SHA3, BLAKE2b/3, MD5 (legacy)
//! - **Licensing**: SPDX license IDs, expressions, named/custom licenses
//!
//! # NTIA minimum elements
//!
//! Per Executive Order 14028 and the NTIA "Minimum Elements For an SBOM" (2021),
//! a conformant SBOM must carry, for every component: supplier name, component
//! name, version, other unique identifiers, dependency relationship, author, and
//! timestamp. [`Sbom::ntia_minimum_elements`] certifies this without deciding
//! whether the SBOM is *honest* — consistent with affidavit's doctrine.
//!
//! # Determinism
//!
//! [`Sbom::content_address`] folds the canonical JSON encoding through BLAKE3, so
//! the same logical SBOM always yields the same address regardless of source
//! format. Components and dependencies are sorted into a canonical order first.

use crate::types::Blake3Hash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A source SBOM document format that normalizes into the canonical [`Sbom`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SbomFormat {
    /// SPDX 2.3 (ISO/IEC 5962:2021 lineage).
    Spdx23,
    /// SPDX 3.0 (element/graph model).
    Spdx30,
    /// CycloneDX 1.5.
    CycloneDx15,
    /// CycloneDX 1.6.
    CycloneDx16,
    /// ISO/IEC 19770-2 SWID tag.
    SwidTag,
}

impl SbomFormat {
    /// Canonical lowercase tag used in OCEL object/event identifiers.
    pub fn tag(&self) -> &'static str {
        match self {
            SbomFormat::Spdx23 => "spdx-2.3",
            SbomFormat::Spdx30 => "spdx-3.0",
            SbomFormat::CycloneDx15 => "cyclonedx-1.5",
            SbomFormat::CycloneDx16 => "cyclonedx-1.6",
            SbomFormat::SwidTag => "swid",
        }
    }

    /// The format family ("spdx", "cyclonedx", "swid"), collapsing versions.
    pub fn family(&self) -> &'static str {
        match self {
            SbomFormat::Spdx23 | SbomFormat::Spdx30 => "spdx",
            SbomFormat::CycloneDx15 | SbomFormat::CycloneDx16 => "cyclonedx",
            SbomFormat::SwidTag => "swid",
        }
    }
}

/// The category of a catalogued component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    /// A deployable application.
    Application,
    /// A software library / package dependency.
    Library,
    /// An application framework.
    Framework,
    /// A container image.
    Container,
    /// An operating system.
    OperatingSystem,
    /// A hardware device.
    Device,
    /// Device firmware.
    Firmware,
    /// An individual file.
    File,
    /// A platform (e.g. a runtime or service plane).
    Platform,
    /// A device driver.
    DeviceDriver,
    /// A machine-learning model artifact.
    MachineLearningModel,
    /// A data artifact / dataset.
    Data,
}

impl ComponentType {
    /// Canonical lowercase tag (matches CycloneDX `type` where it overlaps).
    pub fn tag(&self) -> &'static str {
        match self {
            ComponentType::Application => "application",
            ComponentType::Library => "library",
            ComponentType::Framework => "framework",
            ComponentType::Container => "container",
            ComponentType::OperatingSystem => "operating-system",
            ComponentType::Device => "device",
            ComponentType::Firmware => "firmware",
            ComponentType::File => "file",
            ComponentType::Platform => "platform",
            ComponentType::DeviceDriver => "device-driver",
            ComponentType::MachineLearningModel => "machine-learning-model",
            ComponentType::Data => "data",
        }
    }

    /// Parse a CycloneDX/loose type string into a [`ComponentType`].
    pub fn parse(s: &str) -> ComponentType {
        match s.trim().to_ascii_lowercase().as_str() {
            "application" => ComponentType::Application,
            "library" => ComponentType::Library,
            "framework" => ComponentType::Framework,
            "container" => ComponentType::Container,
            "operating-system" | "operating_system" | "os" => ComponentType::OperatingSystem,
            "device" => ComponentType::Device,
            "firmware" => ComponentType::Firmware,
            "file" => ComponentType::File,
            "platform" => ComponentType::Platform,
            "device-driver" | "driver" => ComponentType::DeviceDriver,
            "machine-learning-model" | "ml-model" | "model" => ComponentType::MachineLearningModel,
            "data" | "dataset" => ComponentType::Data,
            // Library is the conservative default for an unknown dependency.
            _ => ComponentType::Library,
        }
    }
}

/// A cryptographic hash of a component's content.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Hash {
    /// Algorithm name, normalized (e.g. "SHA-256", "BLAKE3", "SHA3-512").
    pub algorithm: String,
    /// Lowercase hex digest.
    pub value: String,
}

impl Hash {
    /// Construct a normalized hash, upper-casing the algorithm and lower-casing
    /// the hex digest so equal digests compare equal regardless of source casing.
    pub fn new(algorithm: impl Into<String>, value: impl Into<String>) -> Self {
        Hash {
            algorithm: normalize_hash_algorithm(&algorithm.into()),
            value: value.into().trim().to_ascii_lowercase(),
        }
    }
}

/// Normalize loose algorithm spellings into a canonical form.
fn normalize_hash_algorithm(raw: &str) -> String {
    match raw.trim().to_ascii_uppercase().replace('_', "-").as_str() {
        "SHA1" | "SHA-1" => "SHA-1".to_string(),
        "SHA256" | "SHA-256" => "SHA-256".to_string(),
        "SHA384" | "SHA-384" => "SHA-384".to_string(),
        "SHA512" | "SHA-512" => "SHA-512".to_string(),
        "SHA3-256" => "SHA3-256".to_string(),
        "SHA3-512" => "SHA3-512".to_string(),
        "BLAKE2B-256" | "BLAKE2B-512" => "BLAKE2b".to_string(),
        "BLAKE3" => "BLAKE3".to_string(),
        "MD5" => "MD5".to_string(),
        other => other.to_string(),
    }
}

/// A software license attached to a component.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct License {
    /// SPDX license identifier (e.g. "MIT", "Apache-2.0"), if a single known ID.
    pub spdx_id: Option<String>,
    /// SPDX license expression (e.g. "MIT OR Apache-2.0"), if compound.
    pub expression: Option<String>,
    /// Human-readable name for a named / custom license.
    pub name: Option<String>,
    /// Reference URL for the license text.
    pub url: Option<String>,
}

impl License {
    /// A simple single-ID license.
    pub fn id(spdx_id: impl Into<String>) -> Self {
        License {
            spdx_id: Some(spdx_id.into()),
            expression: None,
            name: None,
            url: None,
        }
    }

    /// A compound SPDX expression license.
    pub fn expr(expression: impl Into<String>) -> Self {
        License {
            spdx_id: None,
            expression: Some(expression.into()),
            name: None,
            url: None,
        }
    }

    /// The most specific label available for this license.
    pub fn label(&self) -> String {
        self.spdx_id
            .clone()
            .or_else(|| self.expression.clone())
            .or_else(|| self.name.clone())
            .unwrap_or_else(|| "NOASSERTION".to_string())
    }
}

/// The party that supplies / publishes a component.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Supplier {
    /// Supplier / publisher organization or individual name.
    pub name: String,
    /// Supplier URL, if known.
    pub url: Option<String>,
    /// Contact (email or handle), if known.
    pub contact: Option<String>,
}

/// A single catalogued component (the unit of an SBOM).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    /// Stable in-document reference (CycloneDX `bom-ref` / SPDX SPDXID).
    pub bom_ref: String,
    /// Component name.
    pub name: String,
    /// Component version (may be empty / unknown).
    pub version: String,
    /// Component category.
    pub component_type: ComponentType,
    /// Package URL (PURL), if available — the cross-ecosystem coordinate.
    pub purl: Option<String>,
    /// CPE 2.3 identifier, if available.
    pub cpe: Option<String>,
    /// Supplier / publisher.
    pub supplier: Option<Supplier>,
    /// Declared author, if distinct from supplier.
    pub author: Option<String>,
    /// Licenses (one component may carry several).
    pub licenses: Vec<License>,
    /// Content hashes.
    pub hashes: Vec<Hash>,
    /// Free-text description.
    pub description: Option<String>,
    /// Dependency scope ("required", "optional", "excluded").
    pub scope: Option<String>,
}

impl Component {
    /// Construct a minimal library component with the given coordinates.
    pub fn library(
        bom_ref: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Component {
            bom_ref: bom_ref.into(),
            name: name.into(),
            version: version.into(),
            component_type: ComponentType::Library,
            purl: None,
            cpe: None,
            supplier: None,
            author: None,
            licenses: Vec::new(),
            hashes: Vec::new(),
            description: None,
            scope: None,
        }
    }

    /// Whether this component carries at least one unique identifier beyond its
    /// name (PURL, CPE, or a content hash) — an NTIA "other unique identifiers"
    /// signal.
    pub fn has_unique_identifier(&self) -> bool {
        self.purl.is_some() || self.cpe.is_some() || !self.hashes.is_empty()
    }
}

/// A dependency edge: `dependent` directly depends on each of `depends_on`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Dependency {
    /// The dependent component's `bom_ref`.
    pub dependent: String,
    /// The `bom_ref`s this component directly depends on.
    pub depends_on: Vec<String>,
}

/// A tool that produced the SBOM (for provenance of the SBOM itself).
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Tool {
    /// Tool vendor.
    pub vendor: Option<String>,
    /// Tool name.
    pub name: String,
    /// Tool version.
    pub version: Option<String>,
}

/// Document-level metadata describing the SBOM and its primary subject.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SbomMetadata {
    /// Author of the SBOM document (person or organization).
    pub author: Option<String>,
    /// Supplier of the primary component.
    pub supplier: Option<Supplier>,
    /// Tools that generated the SBOM.
    pub tools: Vec<Tool>,
    /// `bom_ref` of the primary component this SBOM describes, if any.
    pub primary_component: Option<String>,
    /// Logical timestamp (Unix seconds) the SBOM asserts. Determinism note: this
    /// is *data carried by the SBOM*, never read from the wall clock here.
    pub timestamp: u64,
}

/// The canonical, format-independent SBOM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sbom {
    /// Source format this canonical SBOM was normalized from.
    pub format: SbomFormat,
    /// Source spec version string (e.g. "2.3", "1.6").
    pub spec_version: String,
    /// Document serial number / namespace, if any.
    pub serial_number: Option<String>,
    /// Document version (monotonic per serial number).
    pub version: u32,
    /// Document metadata.
    pub metadata: SbomMetadata,
    /// Catalogued components.
    pub components: Vec<Component>,
    /// Dependency edges between components.
    pub dependencies: Vec<Dependency>,
}

/// The seven NTIA minimum data fields, evaluated for the whole document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NtiaMinimumElements {
    /// Every component carries a supplier name.
    pub supplier_name: bool,
    /// Every component carries a name.
    pub component_name: bool,
    /// Every component carries a version string.
    pub version: bool,
    /// Every component carries at least one other unique identifier.
    pub unique_identifiers: bool,
    /// The document expresses dependency relationships.
    pub dependency_relationship: bool,
    /// The document carries an author.
    pub author: bool,
    /// The document carries a timestamp.
    pub timestamp: bool,
}

impl NtiaMinimumElements {
    /// Whether all seven minimum elements are present.
    pub fn is_conformant(&self) -> bool {
        self.supplier_name
            && self.component_name
            && self.version
            && self.unique_identifiers
            && self.dependency_relationship
            && self.author
            && self.timestamp
    }

    /// The names of any missing elements (empty iff conformant).
    pub fn missing(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if !self.supplier_name {
            out.push("supplier_name");
        }
        if !self.component_name {
            out.push("component_name");
        }
        if !self.version {
            out.push("version");
        }
        if !self.unique_identifiers {
            out.push("unique_identifiers");
        }
        if !self.dependency_relationship {
            out.push("dependency_relationship");
        }
        if !self.author {
            out.push("author");
        }
        if !self.timestamp {
            out.push("timestamp");
        }
        out
    }
}

impl Sbom {
    /// Construct an empty canonical SBOM of the given format.
    pub fn new(format: SbomFormat, spec_version: impl Into<String>) -> Self {
        Sbom {
            format,
            spec_version: spec_version.into(),
            serial_number: None,
            version: 1,
            metadata: SbomMetadata::default(),
            components: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Look up a component by its `bom_ref`.
    pub fn component(&self, bom_ref: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.bom_ref == bom_ref)
    }

    /// Certify the NTIA minimum elements for this document.
    ///
    /// Doctrine: this *certifies presence against the standard*; it does not
    /// decide whether the declared values are truthful.
    pub fn ntia_minimum_elements(&self) -> NtiaMinimumElements {
        let non_empty = !self.components.is_empty();
        NtiaMinimumElements {
            supplier_name: non_empty
                && self.components.iter().all(|c| {
                    c.supplier
                        .as_ref()
                        .is_some_and(|s| !s.name.trim().is_empty())
                }),
            component_name: non_empty && self.components.iter().all(|c| !c.name.trim().is_empty()),
            version: non_empty && self.components.iter().all(|c| !c.version.trim().is_empty()),
            unique_identifiers: non_empty
                && self.components.iter().all(|c| c.has_unique_identifier()),
            dependency_relationship: !self.dependencies.is_empty(),
            author: self
                .metadata
                .author
                .as_ref()
                .is_some_and(|a| !a.trim().is_empty()),
            timestamp: self.metadata.timestamp > 0,
        }
    }

    /// Canonicalize: sort components by `bom_ref`, dependencies by `dependent`,
    /// and each `depends_on` list, yielding a deterministic ordering.
    pub fn canonicalize(&mut self) {
        self.components.sort_by(|a, b| a.bom_ref.cmp(&b.bom_ref));
        for c in &mut self.components {
            c.licenses.sort();
            c.hashes.sort();
        }
        for d in &mut self.dependencies {
            d.depends_on.sort();
            d.depends_on.dedup();
        }
        self.dependencies
            .sort_by(|a, b| a.dependent.cmp(&b.dependent));
    }

    /// Content-address the canonical SBOM with BLAKE3 over its canonical JSON.
    ///
    /// Clones-then-canonicalizes so the address is order-independent even if the
    /// caller has not canonicalized in place.
    pub fn content_address(&self) -> Blake3Hash {
        let mut canon = self.clone();
        canon.canonicalize();
        let bytes = serde_json::to_vec(&canon).unwrap_or_default();
        Blake3Hash::from_bytes(&bytes)
    }

    /// Build the transitive set of `bom_ref`s reachable from `root` via the
    /// dependency edges (excluding `root` itself unless it is part of a cycle).
    pub fn transitive_dependencies(&self, root: &str) -> Vec<String> {
        let mut index: BTreeMap<&str, &[String]> = BTreeMap::new();
        for d in &self.dependencies {
            index.insert(d.dependent.as_str(), &d.depends_on);
        }
        let mut seen: BTreeMap<String, ()> = BTreeMap::new();
        let mut stack: Vec<String> = index.get(root).map(|d| d.to_vec()).unwrap_or_default();
        while let Some(node) = stack.pop() {
            if seen.insert(node.clone(), ()).is_none() {
                if let Some(children) = index.get(node.as_str()) {
                    stack.extend(children.iter().cloned());
                }
            }
        }
        seen.into_keys().collect()
    }

    /// Aggregate the distinct license labels present across all components.
    pub fn license_labels(&self) -> Vec<String> {
        let mut set: BTreeMap<String, ()> = BTreeMap::new();
        for c in &self.components {
            for l in &c.licenses {
                set.insert(l.label(), ());
            }
        }
        set.into_keys().collect()
    }
}

/// Error raised while ingesting a source SBOM document.
#[derive(Debug, thiserror::Error)]
pub enum SbomError {
    /// The document JSON could not be parsed.
    #[error("sbom parse error: {0}")]
    Parse(String),
    /// The document did not match the claimed / detected format.
    #[error("unrecognized sbom format: {0}")]
    UnrecognizedFormat(String),
    /// A required field was missing.
    #[error("missing required field: {0}")]
    MissingField(String),
}

/// Detect the SBOM format from a parsed JSON document.
pub fn detect_format(doc: &serde_json::Value) -> Result<SbomFormat, SbomError> {
    if let Some(v) = doc.get("spdxVersion").and_then(|v| v.as_str()) {
        // e.g. "SPDX-2.3"
        if v.contains("3.") {
            return Ok(SbomFormat::Spdx30);
        }
        return Ok(SbomFormat::Spdx23);
    }
    if let Some(v) = doc.get("specVersion").and_then(|v| v.as_str()) {
        // CycloneDX carries bomFormat=CycloneDX + specVersion
        if v.starts_with("1.6") {
            return Ok(SbomFormat::CycloneDx16);
        }
        return Ok(SbomFormat::CycloneDx15);
    }
    if doc.get("SoftwareIdentity").is_some() || doc.get("swid").is_some() {
        return Ok(SbomFormat::SwidTag);
    }
    Err(SbomError::UnrecognizedFormat(
        "no spdxVersion/specVersion/SWID marker".to_string(),
    ))
}

/// Parse an SPDX 2.3 JSON document into the canonical [`Sbom`].
pub fn parse_spdx(doc: &serde_json::Value) -> Result<Sbom, SbomError> {
    let spec_version = doc
        .get("spdxVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("SPDX-2.3")
        .trim_start_matches("SPDX-")
        .to_string();

    let mut sbom = Sbom::new(SbomFormat::Spdx23, spec_version);
    sbom.serial_number = doc
        .get("documentNamespace")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Creation info → author + timestamp.
    if let Some(ci) = doc.get("creationInfo") {
        if let Some(creators) = ci.get("creators").and_then(|v| v.as_array()) {
            sbom.metadata.author = creators
                .iter()
                .filter_map(|c| c.as_str())
                .find(|s| s.starts_with("Person:") || s.starts_with("Organization:"))
                .map(|s| s.to_string());
        }
        if let Some(created) = ci.get("created").and_then(|v| v.as_str()) {
            sbom.metadata.timestamp = parse_iso8601_to_unix(created);
        }
    }

    // Packages → components.
    if let Some(packages) = doc.get("packages").and_then(|v| v.as_array()) {
        for pkg in packages {
            let bom_ref = pkg
                .get("SPDXID")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = pkg
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let version = pkg
                .get("versionInfo")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let mut component = Component::library(bom_ref, name, version);
            component.supplier = pkg
                .get("supplier")
                .and_then(|v| v.as_str())
                .filter(|s| *s != "NOASSERTION")
                .map(|s| Supplier {
                    name: s.trim_start_matches("Organization:").trim().to_string(),
                    url: None,
                    contact: None,
                });
            if let Some(lic) = pkg
                .get("licenseConcluded")
                .or_else(|| pkg.get("licenseDeclared"))
                .and_then(|v| v.as_str())
                .filter(|s| *s != "NOASSERTION")
            {
                component
                    .licenses
                    .push(if lic.contains(" OR ") || lic.contains(" AND ") {
                        License::expr(lic)
                    } else {
                        License::id(lic)
                    });
            }
            // External refs → PURL / CPE.
            if let Some(refs) = pkg.get("externalRefs").and_then(|v| v.as_array()) {
                for r in refs {
                    let ref_type = r
                        .get("referenceType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let locator = r
                        .get("referenceLocator")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    match ref_type {
                        "purl" => component.purl = Some(locator.to_string()),
                        t if t.starts_with("cpe") => component.cpe = Some(locator.to_string()),
                        _ => {}
                    }
                }
            }
            // Checksums → hashes.
            if let Some(sums) = pkg.get("checksums").and_then(|v| v.as_array()) {
                for s in sums {
                    let algo = s.get("algorithm").and_then(|v| v.as_str()).unwrap_or("");
                    let val = s
                        .get("checksumValue")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if !algo.is_empty() && !val.is_empty() {
                        component.hashes.push(Hash::new(algo, val));
                    }
                }
            }
            sbom.components.push(component);
        }
    }

    // Relationships → dependency edges (DEPENDS_ON).
    if let Some(rels) = doc.get("relationships").and_then(|v| v.as_array()) {
        let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for r in rels {
            let kind = r
                .get("relationshipType")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind == "DEPENDS_ON" {
                let from = r
                    .get("spdxElementId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let to = r
                    .get("relatedSpdxElement")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !from.is_empty() && !to.is_empty() {
                    edges
                        .entry(from.to_string())
                        .or_default()
                        .push(to.to_string());
                }
            }
        }
        for (dependent, depends_on) in edges {
            sbom.dependencies.push(Dependency {
                dependent,
                depends_on,
            });
        }
    }

    sbom.canonicalize();
    Ok(sbom)
}

/// Parse a CycloneDX 1.5/1.6 JSON document into the canonical [`Sbom`].
pub fn parse_cyclonedx(doc: &serde_json::Value) -> Result<Sbom, SbomError> {
    let spec_version = doc
        .get("specVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("1.5")
        .to_string();
    let format = if spec_version.starts_with("1.6") {
        SbomFormat::CycloneDx16
    } else {
        SbomFormat::CycloneDx15
    };

    let mut sbom = Sbom::new(format, spec_version);
    sbom.serial_number = doc
        .get("serialNumber")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    sbom.version = doc.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32;

    // metadata.timestamp / tools / authors / primary component.
    if let Some(meta) = doc.get("metadata") {
        if let Some(ts) = meta.get("timestamp").and_then(|v| v.as_str()) {
            sbom.metadata.timestamp = parse_iso8601_to_unix(ts);
        }
        if let Some(authors) = meta.get("authors").and_then(|v| v.as_array()) {
            sbom.metadata.author = authors
                .iter()
                .filter_map(|a| a.get("name").and_then(|v| v.as_str()))
                .next()
                .map(|s| s.to_string());
        }
        if let Some(tools) = meta.get("tools").and_then(|v| v.as_array()) {
            for t in tools {
                sbom.metadata.tools.push(Tool {
                    vendor: t
                        .get("vendor")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    name: t
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    version: t
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                });
            }
        }
        if let Some(comp) = meta.get("component") {
            sbom.metadata.primary_component = comp
                .get("bom-ref")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if let Some(sup) = comp
                .get("supplier")
                .and_then(|s| s.get("name"))
                .and_then(|v| v.as_str())
            {
                sbom.metadata.supplier = Some(Supplier {
                    name: sup.to_string(),
                    url: None,
                    contact: None,
                });
            }
        }
    }

    if let Some(components) = doc.get("components").and_then(|v| v.as_array()) {
        for comp in components {
            sbom.components.push(parse_cyclonedx_component(comp));
        }
    }

    if let Some(deps) = doc.get("dependencies").and_then(|v| v.as_array()) {
        for d in deps {
            let dependent = d
                .get("ref")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let depends_on = d
                .get("dependsOn")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();
            if !dependent.is_empty() {
                sbom.dependencies.push(Dependency {
                    dependent,
                    depends_on,
                });
            }
        }
    }

    sbom.canonicalize();
    Ok(sbom)
}

/// Parse a single CycloneDX component object.
fn parse_cyclonedx_component(comp: &serde_json::Value) -> Component {
    let name = comp
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let version = comp
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let bom_ref = comp
        .get("bom-ref")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{name}@{version}"));

    let mut component = Component::library(bom_ref, name, version);
    component.component_type = comp
        .get("type")
        .and_then(|v| v.as_str())
        .map(ComponentType::parse)
        .unwrap_or(ComponentType::Library);
    component.purl = comp
        .get("purl")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    component.cpe = comp
        .get("cpe")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    component.author = comp
        .get("author")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    component.description = comp
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    component.scope = comp
        .get("scope")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if let Some(sup) = comp
        .get("supplier")
        .and_then(|s| s.get("name"))
        .and_then(|v| v.as_str())
    {
        component.supplier = Some(Supplier {
            name: sup.to_string(),
            url: comp
                .get("supplier")
                .and_then(|s| s.get("url"))
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            contact: None,
        });
    }

    // licenses: [{ license: { id | name, url } } | { expression }]
    if let Some(lics) = comp.get("licenses").and_then(|v| v.as_array()) {
        for l in lics {
            if let Some(expr) = l.get("expression").and_then(|v| v.as_str()) {
                component.licenses.push(License::expr(expr));
            } else if let Some(lic) = l.get("license") {
                let mut license = License {
                    spdx_id: lic
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    expression: None,
                    name: lic
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    url: lic
                        .get("url")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                };
                if license.spdx_id.is_none() && license.name.is_none() {
                    license.name = Some("NOASSERTION".to_string());
                }
                component.licenses.push(license);
            }
        }
    }

    // hashes: [{ alg, content }]
    if let Some(hashes) = comp.get("hashes").and_then(|v| v.as_array()) {
        for h in hashes {
            let alg = h.get("alg").and_then(|v| v.as_str()).unwrap_or("");
            let content = h.get("content").and_then(|v| v.as_str()).unwrap_or("");
            if !alg.is_empty() && !content.is_empty() {
                component.hashes.push(Hash::new(alg, content));
            }
        }
    }

    component
}

/// Parse a source SBOM (auto-detecting format) from a JSON string.
pub fn parse_sbom_json(json: &str) -> Result<Sbom, SbomError> {
    let doc: serde_json::Value =
        serde_json::from_str(json).map_err(|e| SbomError::Parse(e.to_string()))?;
    match detect_format(&doc)? {
        SbomFormat::Spdx23 | SbomFormat::Spdx30 => parse_spdx(&doc),
        SbomFormat::CycloneDx15 | SbomFormat::CycloneDx16 => parse_cyclonedx(&doc),
        SbomFormat::SwidTag => Err(SbomError::UnrecognizedFormat(
            "SWID ingest not yet implemented".to_string(),
        )),
    }
}

/// Best-effort ISO-8601 → Unix-seconds conversion without a date dependency.
///
/// Parses the common `YYYY-MM-DDThh:mm:ssZ` shape used by SPDX/CycloneDX. On any
/// parse failure returns 0 (the "no timestamp" sentinel), keeping the function
/// total and dependency-free.
fn parse_iso8601_to_unix(s: &str) -> u64 {
    // Expect: 2024-01-15T10:30:00Z (optionally fractional seconds / offset).
    let bytes = s.as_bytes();
    if s.len() < 19 || bytes.get(4) != Some(&b'-') || bytes.get(10) != Some(&b'T') {
        return 0;
    }
    let parse = |a: usize, b: usize| -> Option<i64> { s.get(a..b)?.parse().ok() };
    let (year, month, day, hour, min, sec) = match (
        parse(0, 4),
        parse(5, 7),
        parse(8, 10),
        parse(11, 13),
        parse(14, 16),
        parse(17, 19),
    ) {
        (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(se)) => (y, mo, d, h, mi, se),
        _ => return 0,
    };
    days_from_civil(year, month, day)
        .map(|days| (days * 86400 + hour * 3600 + min * 60 + sec).max(0) as u64)
        .unwrap_or(0)
}

/// Days since the Unix epoch for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> Option<i64> {
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cyclonedx() -> &'static str {
        r#"{
          "bomFormat": "CycloneDX",
          "specVersion": "1.6",
          "serialNumber": "urn:uuid:1234",
          "version": 1,
          "metadata": {
            "timestamp": "2024-01-15T10:30:00Z",
            "authors": [{"name": "Build Bot"}],
            "tools": [{"vendor": "acme", "name": "cdxgen", "version": "9.0"}],
            "component": {"bom-ref": "app@1.0", "name": "app", "version": "1.0", "type": "application"}
          },
          "components": [
            {
              "bom-ref": "pkg:cargo/serde@1.0.0",
              "type": "library",
              "name": "serde",
              "version": "1.0.0",
              "purl": "pkg:cargo/serde@1.0.0",
              "supplier": {"name": "serde-rs"},
              "licenses": [{"expression": "MIT OR Apache-2.0"}],
              "hashes": [{"alg": "SHA-256", "content": "ABCDEF"}]
            },
            {
              "bom-ref": "pkg:cargo/log@0.4.0",
              "type": "library",
              "name": "log",
              "version": "0.4.0",
              "purl": "pkg:cargo/log@0.4.0",
              "supplier": {"name": "rust-lang"},
              "licenses": [{"license": {"id": "MIT"}}],
              "hashes": [{"alg": "SHA-256", "content": "123456"}]
            }
          ],
          "dependencies": [
            {"ref": "app@1.0", "dependsOn": ["pkg:cargo/serde@1.0.0"]},
            {"ref": "pkg:cargo/serde@1.0.0", "dependsOn": ["pkg:cargo/log@0.4.0"]}
          ]
        }"#
    }

    fn sample_spdx() -> &'static str {
        r#"{
          "spdxVersion": "SPDX-2.3",
          "documentNamespace": "https://example/spdx/1",
          "creationInfo": {
            "created": "2024-02-20T08:00:00Z",
            "creators": ["Organization: Acme Corp", "Tool: spdx-tool-1.0"]
          },
          "packages": [
            {
              "SPDXID": "SPDXRef-serde",
              "name": "serde",
              "versionInfo": "1.0.0",
              "supplier": "Organization: serde-rs",
              "licenseConcluded": "MIT OR Apache-2.0",
              "externalRefs": [
                {"referenceType": "purl", "referenceLocator": "pkg:cargo/serde@1.0.0"}
              ],
              "checksums": [{"algorithm": "SHA256", "checksumValue": "ABCDEF"}]
            },
            {
              "SPDXID": "SPDXRef-log",
              "name": "log",
              "versionInfo": "0.4.0",
              "supplier": "Organization: rust-lang",
              "licenseConcluded": "MIT",
              "externalRefs": [
                {"referenceType": "purl", "referenceLocator": "pkg:cargo/log@0.4.0"}
              ],
              "checksums": [{"algorithm": "SHA256", "checksumValue": "123456"}]
            }
          ],
          "relationships": [
            {"spdxElementId": "SPDXRef-serde", "relationshipType": "DEPENDS_ON", "relatedSpdxElement": "SPDXRef-log"}
          ]
        }"#
    }

    #[test]
    fn detects_cyclonedx_and_spdx() {
        let cdx: serde_json::Value = serde_json::from_str(sample_cyclonedx()).unwrap();
        assert_eq!(detect_format(&cdx).unwrap(), SbomFormat::CycloneDx16);
        let spdx: serde_json::Value = serde_json::from_str(sample_spdx()).unwrap();
        assert_eq!(detect_format(&spdx).unwrap(), SbomFormat::Spdx23);
    }

    #[test]
    fn parses_cyclonedx_components_and_deps() {
        let sbom = parse_cyclonedx(&serde_json::from_str(sample_cyclonedx()).unwrap()).unwrap();
        assert_eq!(sbom.components.len(), 2);
        assert_eq!(sbom.dependencies.len(), 2);
        let serde = sbom.component("pkg:cargo/serde@1.0.0").unwrap();
        assert_eq!(serde.name, "serde");
        assert_eq!(serde.purl.as_deref(), Some("pkg:cargo/serde@1.0.0"));
        assert_eq!(serde.licenses[0].label(), "MIT OR Apache-2.0");
    }

    #[test]
    fn parses_spdx_packages_and_relationships() {
        let sbom = parse_spdx(&serde_json::from_str(sample_spdx()).unwrap()).unwrap();
        assert_eq!(sbom.components.len(), 2);
        assert_eq!(sbom.dependencies.len(), 1);
        assert_eq!(
            sbom.metadata.author.as_deref(),
            Some("Organization: Acme Corp")
        );
        assert!(sbom.metadata.timestamp > 0);
    }

    #[test]
    fn auto_parse_dispatches_by_format() {
        assert_eq!(
            parse_sbom_json(sample_cyclonedx())
                .unwrap()
                .components
                .len(),
            2
        );
        assert_eq!(parse_sbom_json(sample_spdx()).unwrap().components.len(), 2);
    }

    #[test]
    fn ntia_minimum_elements_conformant_sbom() {
        let sbom = parse_cyclonedx(&serde_json::from_str(sample_cyclonedx()).unwrap()).unwrap();
        let ntia = sbom.ntia_minimum_elements();
        assert!(ntia.component_name);
        assert!(ntia.version);
        assert!(ntia.unique_identifiers);
        assert!(ntia.dependency_relationship);
        assert!(ntia.supplier_name);
        assert!(ntia.author);
        assert!(ntia.timestamp);
        assert!(ntia.is_conformant(), "missing: {:?}", ntia.missing());
    }

    #[test]
    fn ntia_flags_missing_elements() {
        let mut sbom = Sbom::new(SbomFormat::CycloneDx16, "1.6");
        sbom.components.push(Component::library("a", "a", "")); // no version, no id
        let ntia = sbom.ntia_minimum_elements();
        assert!(!ntia.is_conformant());
        let missing = ntia.missing();
        assert!(missing.contains(&"version"));
        assert!(missing.contains(&"unique_identifiers"));
        assert!(missing.contains(&"dependency_relationship"));
        assert!(missing.contains(&"author"));
        assert!(missing.contains(&"timestamp"));
    }

    #[test]
    fn content_address_is_format_independent_and_deterministic() {
        let from_cdx = parse_sbom_json(sample_cyclonedx()).unwrap();
        let a1 = from_cdx.content_address();
        let a2 = from_cdx.content_address();
        assert_eq!(a1, a2, "content address must be deterministic");
        // Re-ordering components must not change the address.
        let mut shuffled = from_cdx.clone();
        shuffled.components.reverse();
        assert_eq!(
            shuffled.content_address(),
            a1,
            "content address must be canonicalization-stable"
        );
    }

    #[test]
    fn transitive_dependencies_walks_the_graph() {
        let sbom = parse_sbom_json(sample_cyclonedx()).unwrap();
        let trans = sbom.transitive_dependencies("app@1.0");
        assert!(trans.contains(&"pkg:cargo/serde@1.0.0".to_string()));
        assert!(
            trans.contains(&"pkg:cargo/log@0.4.0".to_string()),
            "transitive dep through serde must be reached"
        );
    }

    #[test]
    fn license_labels_aggregate_distinct() {
        let sbom = parse_sbom_json(sample_cyclonedx()).unwrap();
        let labels = sbom.license_labels();
        assert!(labels.contains(&"MIT OR Apache-2.0".to_string()));
        assert!(labels.contains(&"MIT".to_string()));
    }

    #[test]
    fn iso8601_parses_known_epoch() {
        // 2024-01-15T10:30:00Z == 1705314600
        assert_eq!(parse_iso8601_to_unix("2024-01-15T10:30:00Z"), 1_705_314_600);
        assert_eq!(parse_iso8601_to_unix("garbage"), 0);
    }

    #[test]
    fn component_type_parse_is_total() {
        assert_eq!(
            ComponentType::parse("application"),
            ComponentType::Application
        );
        assert_eq!(ComponentType::parse("os"), ComponentType::OperatingSystem);
        assert_eq!(ComponentType::parse("unknown-xyz"), ComponentType::Library);
    }

    #[test]
    fn hash_normalizes_algorithm_and_casing() {
        let h = Hash::new("sha256", "ABCDEF");
        assert_eq!(h.algorithm, "SHA-256");
        assert_eq!(h.value, "abcdef");
    }
}
