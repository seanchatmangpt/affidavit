# Affidavit Documentation Index

**Main Project**: Provenance Layer — Receipt Assembly & Certification

---

## 📖 Essential Documentation

Start here for project overview and setup:

- **[README.md](../README.md)** — Quick start, doctrine, CLI surface, worked example
- **[CLAUDE.md](../CLAUDE.md)** — Complete project guide, architecture, development workflow
- **[CHANGELOG.md](../CHANGELOG.md)** — Version history and breaking changes
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** — Contribution guidelines
- **[STATUS.md](../STATUS.md)** — Current project status and releases
- **[IMPLEMENTATION_SUMMARY.md](../IMPLEMENTATION_SUMMARY.md)** — Western Electric quality monitoring implementation details

---

## 🔌 Integration Guides

Documentation for ecosystem integrations:

### Language Server Protocol (LSP)
- [Architecture](integrations/LSP_MAX_INTEGRATION_ARCHITECTURE.md)
- [Code Templates](integrations/LSP_MAX_INTEGRATION_CODE_TEMPLATES.md)
- [Implementation Plan](integrations/LSP_MAX_INTEGRATION_PLAN.md)
- [Quick Reference](integrations/LSP_MAX_INTEGRATION_QUICK_REFERENCE.md)
- [Summary](integrations/LSP_MAX_INTEGRATION_SUMMARY.md)
- [Index](integrations/LSP_MAX_INTEGRATION_INDEX.md)

### Process Mining (WASM4PM)
- [Integration Plan](integrations/WASM4PM_INTEGRATION_PLAN.md)
- [Integration Summary](integrations/WASM4PM_INTEGRATION_SUMMARY.md)
- [Quick Reference](integrations/WASM4PM_QUICK_REFERENCE.md)
- [80/20 Breakdown](integrations/WASM4PM_80_20_BREAKDOWN.md)
- [Witness Test Templates](integrations/WASM4PM_WITNESS_TEST_TEMPLATES.md)
- [Index](integrations/WASM4PM_INDEX.md)

### Canonicalization (CLNRM)
- [Integration Plan](integrations/CLNRM_INTEGRATION_PLAN_26.6.17.md)
- [Integration README](integrations/CLNRM_INTEGRATION_README.md)
- [Integration Examples](integrations/CLNRM_INTEGRATION_EXAMPLES.md)
- [Public API Surface](integrations/CLNRM_PUBLIC_API_SURFACE.md)

### Other Integrations
- [General Integrations Overview](integrations/INTEGRATIONS.md)

---

## 📊 Western Electric Quality Monitoring

Real-time code quality monitoring using statistical process control:

- **Theory & Reference**: See [Western Electric Complete Guide](WESTERN_ELECTRIC_COMPLETE.md)
- **Quick Index**: See [Western Electric Index](WESTERN_ELECTRIC_INDEX.md)
- **Benchmarking Guide**: See [benchmarks/README_QUALITY_WESTERN_ELECTRIC.md](../benches/README_QUALITY_WESTERN_ELECTRIC.md)

---

## 📚 Learning Resources

### Understanding Affidavit
1. Read **README.md** for the elevator pitch
2. Review **CLAUDE.md** for complete architecture
3. Work through examples in **examples/** directory

### For Contributors
1. Check **CONTRIBUTING.md**
2. Review relevant integration guides in **integrations/**
3. See **CLAUDE.md** for development workflow

### For Integration Work
1. See integration guide relevant to your ecosystem (LSP, WASM4PM, CLNRM)
2. Check code templates and examples
3. Refer to quick reference guides

---

## 🗂️ Documentation Structure

```
affidavit/
├── README.md                      # Main project documentation
├── CLAUDE.md                      # Complete project guide
├── CHANGELOG.md                   # Version history
├── CONTRIBUTING.md                # Contribution guidelines
├── STATUS.md                      # Project status
├── IMPLEMENTATION_SUMMARY.md      # Western Electric implementation
├── docs/
│   ├── INDEX.md                   # This file
│   ├── WESTERN_ELECTRIC_COMPLETE.md    # Theory & reference
│   ├── WESTERN_ELECTRIC_INDEX.md       # Quick navigation
│   ├── integrations/              # Integration documentation
│   │   ├── LSP_MAX_INTEGRATION_*.md
│   │   ├── WASM4PM_*.md
│   │   ├── CLNRM_INTEGRATION_*.md
│   │   └── INTEGRATIONS.md
│   ├── archive/                   # Archived planning documents
│   └── (other reference docs)
├── benches/                       # Benchmarking documentation
├── src/                           # Source code
├── examples/                      # Working examples
└── tests/                         # Test suites
```

---

## 🚀 Quick Links

**Getting Started**
- [README.md](../README.md) — Start here
- [CLAUDE.md](../CLAUDE.md) — Full guide
- [examples/golden_run.sh](../examples/golden_run.sh) — Full lifecycle demo

**Key Concepts**
- Receipt chain and sealing
- 7-stage verification pipeline
- OCEL integration
- Western Electric rules (7 SPC rules)

**Development**
- [CONTRIBUTING.md](../CONTRIBUTING.md) — How to contribute
- [CLAUDE.md](../CLAUDE.md) — Development workflow
- [STATUS.md](../STATUS.md) — Current priorities

**Integration**
- [LSP Integration](integrations/LSP_MAX_INTEGRATION_INDEX.md)
- [Process Mining](integrations/WASM4PM_INDEX.md)
- [Canonicalization](integrations/CLNRM_INTEGRATION_README.md)

---

## 📝 Notes

- **Archived Documents**: Historical planning and design docs are in `docs/archive/`
- **Integration Guides**: All ecosystem integration documentation is in `docs/integrations/`
- **Code Examples**: Working examples are in `examples/` directory
- **Tests**: Comprehensive test suite in `tests/` directory
- **Benchmarks**: Performance benchmarks and analysis in `benches/` directory

---

## 🔍 Search Tips

Looking for information about:
- **Receipt verification**: See CLAUDE.md (7-stage pipeline section)
- **Event emission**: See CLAUDE.md (CLI Surface section)
- **Integration**: See relevant folder in `docs/integrations/`
- **Code quality monitoring**: See IMPLEMENTATION_SUMMARY.md
- **Benchmarking**: See benches/README_QUALITY_WESTERN_ELECTRIC.md

---

*Last Updated: 2026-06-17*
