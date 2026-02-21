# VASP Trace Output System Analysis

**Investigation:** Trace Semantics Engine — DSL Trace Format Survey (VASP)
**Date:** 2026-02-20
**Scope:** Comprehensive analysis of VASP's output system for ATHENA IR design
**Architecture References:** ARCHITECTURE.md §4.5, §5.3; VISION.md §4.1, Open Question #1

**Evidence Basis:** This analysis draws on documented VASP Wiki content, pymatgen/ASE API documentation, published VASP tutorials, and the author's domain knowledge of DFT workflows. Each claim is tagged with its evidence basis: **[documented]** = present in VASP Wiki or official documentation; **[observed]** = widely reported in community usage/tutorials; **[inferred]** = reasoned from documented behavior but not explicitly stated in official sources.

---

## 1. Output File Inventory

VASP produces a well-defined set of output files during DFT calculations. The following catalog covers the primary and secondary output files relevant to ATHENA's trace semantics requirements.

### 1.1 Primary Output Files

| File | Content | Format | Typical Size | Programmatic Access | Layer |
|:---|:---|:---|:---|:---|:---|
| **OUTCAR** | Comprehensive calculation log: all INCAR parameters (echoed), symmetry analysis, k-point generation, SCF iteration details (energy, charge mixing), forces, stress tensor, timing, memory usage, warnings | Text (unstructured) | 1 MB–1 GB+ | pymatgen `Outcar`, ASE parsers, custom regex | Mixed (theory + implementation) |
| **OSZICAR** | Per-SCF-step and per-ionic-step summary: iteration count, total energy, energy change (dE), residual norm (d eps), charge density mixing metric (ncg) | Text (structured, columnar) | 1–100 KB | pymatgen `Oszicar`, trivial line parsing | Primarily theory (convergence trajectory) |
| **vasprun.xml** | Machine-readable XML record: input parameters, crystal structure (initial/final), k-points, eigenvalues, DOS, forces, stress, total energy per ionic step, dielectric properties, Born effective charges | XML (structured) | 10 MB–1 GB+ | pymatgen `Vasprun`, ASE `read_vasp_xml`, lxml/ElementTree | Mixed (theory + implementation, but cleanly separated within XML) |
| **CONTCAR** | Final (or last-written) ionic positions in POSCAR format; the relaxed structure | Text (POSCAR format) | 1–10 KB | pymatgen `Structure.from_file`, ASE `read_vasp` | Theory (structural result) |

### 1.2 Electronic Structure Output Files

| File | Content | Format | Typical Size | Programmatic Access | Layer |
|:---|:---|:---|:---|:---|:---|
| **EIGENVAL** | Eigenvalues (band energies) at each k-point for each band and spin channel | Text (structured, columnar) | 100 KB–10 MB | pymatgen `Eigenval`, custom parsers | Theory (electronic structure result) |
| **DOSCAR** | Total and projected density of states; integrated DOS; Fermi energy | Text (structured, columnar) | 100 KB–100 MB | pymatgen `Doscar`, ASE | Theory (electronic structure result) |
| **PROCAR** | Projected band structure: orbital-resolved, site-resolved, spin-resolved contributions to each band at each k-point | Text (structured, columnar) | 10 MB–1 GB | pymatgen `Procar` | Theory (electronic structure decomposition) |
| **CHGCAR** | Charge density on the FFT grid; includes augmentation charges in PAW spheres | Binary-like text (header + volumetric data) | 10 MB–10 GB | pymatgen `Chgcar`, VESTA, custom parsers | Theory (electron density result) |
| **WAVECAR** | Plane-wave coefficients for all bands at all k-points; full wavefunction | Binary (Fortran unformatted) | 100 MB–100 GB | pymatgen (limited), WaveTrans, custom Fortran readers | Theory (complete quantum state), but also implementation (basis set representation) |
| **LOCPOT** | Local potential on the FFT grid (electrostatic potential) | Binary-like text (header + volumetric data) | 10 MB–10 GB | pymatgen `Locpot`, VESTA | Theory (electrostatic result) |

### 1.3 Auxiliary Output Files

| File | Content | Format | Typical Size | Programmatic Access | Layer |
|:---|:---|:---|:---|:---|:---|
| **IBZKPT** | Irreducible k-points generated from KPOINTS input, with weights | Text (structured) | 1–100 KB | pymatgen, manual parsing | Theory (Brillouin zone sampling) |
| **XDATCAR** | Trajectory of ionic positions across all ionic steps (MD or relaxation) | Text (multi-frame POSCAR) | 10 KB–100 MB | pymatgen `Xdatcar`, ASE trajectory reader | Theory (structural trajectory) |
| **PCDAT** | Pair correlation function (MD runs) | Text (columnar) | 1–10 KB | Custom parsers | Theory (statistical mechanics observable) |
| **REPORT** | Extended logging (VASP 6+); detailed SCF convergence, subspace rotation, Davidson steps | Text | 1–100 MB | Custom parsers (no standard library support) | Mixed (heavy implementation detail) |
| **STOPCAR** | Created by user to signal graceful stop; not a true output | Text (flag) | <1 KB | N/A | Implementation (control signal) |
| **vasp.out / stdout** | Standard output stream: timing, memory allocation, MPI rank info, parallelization summary, warnings, error messages | Text (unstructured) | 10 KB–100 MB | Custom parsers, regex | Primarily implementation |

### 1.4 Completeness Assessment

**[Observed]** The combination of vasprun.xml + OUTCAR + OSZICAR provides a nearly complete record of a VASP calculation's theory-layer behavior. However:
- vasprun.xml does not capture all warnings and diagnostic messages present in OUTCAR. **[Documented]**
- OUTCAR contains implementation timing data (per-subroutine timings) not present in vasprun.xml. **[Documented]**
- WAVECAR and CHGCAR contain the full quantum mechanical state but are binary/volumetric and difficult to parse for causal analysis. **[Observed]**
- Standard output (stdout/vasp.out) contains critical parallelization and memory information not replicated elsewhere. **[Observed]**

---

## 2. Theory-Implementation API Boundary

VASP's primary input specification is the INCAR file (supplemented by POSCAR, POTCAR, KPOINTS). The INCAR tags can be classified into theory-layer and implementation-layer parameters, though the boundary is not perfectly clean.

### 2.1 Theory Layer Parameters

These INCAR tags specify the physics of the calculation. Changing them changes the scientific question being asked.

| Tag | Function | Theory Content |
|:---|:---|:---|
| **GGA** | Exchange-correlation functional (PBE, PW91, RPBE, etc.) | Selects the approximation to exact exchange-correlation |
| **METAGGA** | Meta-GGA functional (SCAN, TPSS, r2SCAN, etc.) | Higher-rung DFT approximation |
| **LHFCALC** | Enable exact (Hartree-Fock) exchange | Hybrid functional flag |
| **AEXX** | Fraction of exact exchange in hybrid functionals | Theory parameter (e.g., 0.25 for PBE0) |
| **ENCUT** | Plane-wave energy cutoff (eV) | Basis set completeness; directly affects accuracy |
| **ISMEAR** | Smearing method (Gaussian, Methfessel-Paxton, tetrahedron) | Brillouin zone integration scheme |
| **SIGMA** | Smearing width (eV) | Controls electronic temperature / BZ integration |
| **EDIFF** | SCF convergence criterion (eV) | Precision of the self-consistent solution |
| **EDIFFG** | Ionic convergence criterion (eV/Angstrom for forces) | Precision of structural relaxation |
| **IBRION** | Ionic relaxation algorithm (conjugate gradient, quasi-Newton, MD) | Method for exploring potential energy surface |
| **ISIF** | Stress tensor and cell shape/volume relaxation control | What degrees of freedom are optimized |
| **NSW** | Maximum number of ionic steps | Bound on structural optimization |
| **LDAU** | DFT+U correction: on-site Coulomb interaction | Hubbard U correction (strong correlation physics) |
| **LDAUU / LDAUJ** | U and J values for DFT+U | Specific on-site interaction parameters |
| **LDAUTYPE** | DFT+U formalism (Dudarev, Liechtenstein) | Method for applying Hubbard correction |
| **IVDW** | Van der Waals correction method (DFT-D2, D3, TS, etc.) | Dispersion interaction treatment |
| **LSOC** | Spin-orbit coupling | Relativistic effect inclusion |
| **ISPIN** | Spin polarization (1=non-magnetic, 2=spin-polarized) | Magnetic treatment |
| **MAGMOM** | Initial magnetic moments per atom | Starting magnetic configuration |
| **LORBIT** | Projected DOS / band decomposition method | Controls electronic analysis output |
| **NEDOS** | Number of DOS grid points | Resolution of density of states |
| **NBANDS** | Number of electronic bands | Size of eigenvalue problem |

**Theory is also distributed across input files [documented]:**
- **POSCAR**: Crystal structure (lattice vectors, atomic positions, species). This is theory — it defines the physical system.
- **POTCAR**: Pseudopotentials (PAW datasets). This is theory — it defines the electron-ion interaction approximation. The choice of POTCAR (e.g., standard vs. _sv vs. _pv) determines which electrons are treated as valence.
- **KPOINTS**: Brillouin zone sampling mesh. This is theory — it determines the completeness of reciprocal-space integration.

### 2.2 Implementation Layer Parameters

These INCAR tags control how the calculation is executed without (in principle) changing the physical result.

| Tag | Function | Implementation Content |
|:---|:---|:---|
| **NCORE / NPAR** | Parallelization over bands (NCORE = cores per orbital band group) | MPI/OpenMP work distribution |
| **KPAR** | Parallelization over k-points | K-point parallel decomposition |
| **LPLANE** | Plane-wise data distribution in FFT | FFT parallelization strategy |
| **NCORE** | Number of cores working on individual orbital | Band-level parallelism |
| **LWAVE** | Write WAVECAR (T/F) | Output control (does not affect physics) |
| **LCHARG** | Write CHGCAR (T/F) | Output control |
| **LELF** | Write ELFCAR (T/F) | Output control |
| **ALGO** | Electronic minimization algorithm (Normal, Fast, Very_Fast, All, Damped) | SCF solver strategy |
| **NELM** | Maximum SCF iterations | Bound on electronic minimization |
| **NELMIN** | Minimum SCF iterations | Lower bound on electronic minimization |
| **LREAL** | Real-space projection (Auto, T, F) | Projection operator evaluation domain |
| **NSIM** | Number of bands treated simultaneously in blocked Davidson | Blocking parameter for eigenvalue solver |
| **ISTART** | How to initialize wavefunctions (0=scratch, 1=from WAVECAR) | Restart control |
| **ICHARG** | How to initialize charge density (0=from wavefunctions, 1=from CHGCAR, 2=superposition) | Initialization strategy |

### 2.3 Ambiguous Parameters

These parameters affect both physics and performance in non-trivial ways. This is a critical finding for ATHENA.

| Tag | Theory Aspect | Implementation Aspect | Nature of Ambiguity |
|:---|:---|:---|:---|
| **PREC** (Accurate, Normal, Low) | Sets ENCUT default, FFT grid density, augmentation grid density — all affect numerical accuracy of physical results | Controls memory allocation, FFT grid sizes, wrap-around errors | A single tag that simultaneously configures theory precision and implementation resource allocation. **[Documented]** |
| **LREAL** (Auto, True, False) | Real-space projection introduces controlled approximation errors in forces | Real-space projection is faster for large cells but less accurate | Trades physical accuracy for speed. For small cells, LREAL=.FALSE. is required for correct physics. **[Documented]** |
| **ADDGRID** | Finer integration grid improves augmentation charge accuracy | Increases memory and computation time | Pure accuracy-cost tradeoff. **[Documented]** |
| **ALGO** | Different algorithms may converge to different local SCF minima for difficult systems (e.g., magnetic, strongly correlated) | Different algorithms have different computational cost and stability | For most systems, ALGO is purely implementation. For pathological cases (multiple SCF minima), it can affect which physical solution is found. **[Observed]** |
| **ENCUT** (revisited) | Basis set completeness — fundamental physical accuracy | Higher ENCUT = more plane waves = more memory and time | Listed under theory, but its practical value is often chosen based on computational budget. The "converged" value is theory; the actual value used may be a compromise. **[Observed]** |
| **NBANDS** (revisited) | Must be large enough to capture physical states | Larger NBANDS = more memory and eigenvalue solver work | Too few bands causes unphysical results. Excess bands are purely computational cost. **[Observed]** |

### 2.4 Boundary Cleanliness Assessment

**Is VASP's theory-implementation boundary clean enough for ATHENA?**

**Strengths:**
1. INCAR provides a single-file specification point where most parameters are declared. This is far more centralized than, say, a custom Python MD script. **[Documented]**
2. The POTCAR + POSCAR + KPOINTS + INCAR quartet forms a complete, declarative specification of the calculation. There is no imperative code in the user-facing input. **[Documented]**
3. vasprun.xml echoes all input parameters and tags them with their effective values (including defaults), making post-hoc reconstruction of the full specification possible. **[Documented]**

**Weaknesses:**
1. The theory-implementation boundary is not formally declared by VASP. There is no VASP-provided metadata that labels tags as "physics" vs. "execution." The classification above is domain knowledge, not API-enforced. **[Inferred]**
2. Ambiguous parameters (PREC, LREAL, ALGO) create genuine cross-layer coupling. A change to PREC intended to save memory may inadvertently change the physical result. **[Documented]**
3. Theory is distributed across four input files (INCAR, POSCAR, POTCAR, KPOINTS), not one. The POTCAR choice (which electrons are valence) is a theory decision that is often implicit in workflow tools rather than explicit in the INCAR. **[Documented]**
4. VASP's defaults are version-dependent. Some defaults changed between VASP 5 and VASP 6 (e.g., ALGO, LREAL behavior). The same INCAR may produce different physics in different VASP versions. **[Observed]**

**Comparison with MD codes (OpenMM, GROMACS):**
- MD codes have a cleaner separation because force fields are external data files (topology + parameter files), completely decoupled from the engine. In VASP, the "force field" equivalent (the exchange-correlation functional) is selected by an INCAR tag and compiled into the binary — there is no user-accessible force field file. **[Inferred]**
- MD codes expose simulation parameters (timestep, thermostat) and engine parameters (GPU/CPU, PME grid) as distinct configuration sections. VASP mixes both in a single flat INCAR namespace. **[Observed]**
- However, VASP's advantage is that its input is purely declarative — there is no scripting in the input, whereas OpenMM requires Python scripting to configure simulations. This makes VASP's input more amenable to static analysis. **[Inferred]**

**Verdict for ATHENA:** VASP's boundary is usable but requires an explicit classification layer that VASP itself does not provide. ATHENA's DSL Environment Interface for VASP would need to maintain a tag-level metadata table classifying each INCAR parameter, each POTCAR variant, and each KPOINTS specification mode into theory/implementation/ambiguous categories. This is a finite, maintainable engineering task — the INCAR tag set is documented and stable (approximately 200-300 tags total, of which perhaps 50-80 are commonly used). **[Inferred]**

---

## 3. vasprun.xml Structure

The vasprun.xml file is VASP's primary machine-readable output. It is the most important file for ATHENA's trace semantics because it is structured, complete for most purposes, and well-supported by parsing libraries.

### 3.1 XML Hierarchy

**[Documented]** The vasprun.xml file has the following top-level structure:

```xml
<modeling>
  <generator>         <!-- VASP version, date, platform -->
  <incar>             <!-- All INCAR parameters with effective values -->
  <kpoints>           <!-- K-point specification and generated mesh -->
  <parameters>        <!-- Detailed parameter set (includes defaults) -->
  <atominfo>          <!-- Atom types, counts, pseudopotential info -->
  <structure name="initialpos">  <!-- Initial crystal structure -->

  <calculation>       <!-- One per ionic step -->
    <scstep>          <!-- One per SCF step within ionic step -->
      <energy>        <!-- Energy components per SCF step -->
      <time>          <!-- Wall/CPU time per SCF step -->
    </scstep>
    ...
    <structure>       <!-- Structure after this ionic step -->
    <varray name="forces">    <!-- Forces on atoms -->
    <varray name="stress">    <!-- Stress tensor -->
    <energy>          <!-- Final energy for this ionic step -->
    <eigenvalues>     <!-- Band energies at k-points -->
    <dos>             <!-- Density of states (final step only) -->
    <projected>       <!-- Projected DOS/bands (if requested) -->
  </calculation>
  ...                 <!-- Repeated for each ionic step -->

  <structure name="finalpos">  <!-- Final crystal structure -->
</modeling>
```

### 3.2 Theory vs. Implementation Content Separation

| XML Node | Content | Layer | IR Relevance |
|:---|:---|:---|:---|
| `<generator>` | VASP version, compilation date, platform | Implementation | Version tracking for reproducibility |
| `<incar>` | All INCAR parameters (user-set + defaults) | Mixed (see §2) | Full specification reconstruction |
| `<kpoints>` | K-point generation method, mesh, weights | Theory | Brillouin zone sampling specification |
| `<parameters>` | Complete parameter set including internal defaults | Mixed | Superset of `<incar>` with resolved defaults |
| `<atominfo>` | Species, atom counts, pseudopotential names, valence electron counts | Theory | System definition and PAW dataset identity |
| `<structure>` | Lattice vectors, atomic positions, selective dynamics flags | Theory | Physical system state |
| `<calculation>/<scstep>/<energy>` | Per-SCF energy decomposition (alphaZ, ewald, hartreedc, XC, etc.) | Theory | SCF convergence trajectory; energy decomposition enables fault analysis |
| `<calculation>/<scstep>/<time>` | CPU and wall time per SCF step | Implementation | Performance profiling |
| `<calculation>/<varray name="forces">` | Cartesian forces on each atom | Theory | Forces are the primary observable for ionic relaxation convergence |
| `<calculation>/<varray name="stress">` | 3x3 stress tensor (kBar) | Theory | Cell relaxation observable |
| `<calculation>/<energy>` | Final energy components per ionic step | Theory | Primary thermodynamic result |
| `<calculation>/<eigenvalues>` | Band energies at k-points per spin channel | Theory | Electronic structure result |
| `<calculation>/<dos>` | Total and projected DOS | Theory | Electronic structure analysis |

### 3.3 Completeness for IR Purposes

**What vasprun.xml captures [documented]:**
- Complete input specification (all parameters with effective values)
- Full SCF convergence trajectory (energy per electronic step)
- Ionic relaxation trajectory (structure, energy, forces, stress per ionic step)
- Eigenvalues and DOS (final step, or per ionic step if LORBIT is set)
- Timing information per SCF step

**What vasprun.xml does NOT capture [observed/inferred]:**
- Warnings and diagnostic messages (these appear only in OUTCAR and stdout)
- Detailed charge mixing diagnostics (RMM-DIIS residuals, subspace rotation info)
- Memory allocation details
- MPI communication patterns and parallelization efficiency
- Symmetry operations applied (partially in OUTCAR)
- POTCAR checksums/details (partially present via `<atominfo>`, but not the full pseudopotential provenance)
- Error messages and crash diagnostics (these go to stdout/stderr)
- Per-subroutine timing breakdown (only in OUTCAR)

**Assessment:** vasprun.xml is approximately 80-90% complete for theory-layer trace reconstruction. The missing 10-20% is primarily implementation-layer diagnostics (warnings, memory, parallelization) and some theory-layer metadata (full POTCAR provenance, symmetry operations). For ATHENA's three-stage audit, vasprun.xml alone is insufficient — OUTCAR and stdout are needed for the implementation audit (Stage 1). **[Inferred]**

---

## 4. OUTCAR vs. OSZICAR vs. vasprun.xml

### 4.1 Content Comparison

| Feature | OUTCAR | OSZICAR | vasprun.xml |
|:---|:---|:---|:---|
| **Input parameters** | Full echo with explanations | No | Full echo (XML structured) |
| **SCF energy per step** | Yes (verbose) | Yes (compact summary) | Yes (structured XML) |
| **SCF convergence metrics** | Yes (dE, RMS, charge mixing) | Yes (dE, d eps, ncg) | Yes (energy only) |
| **Ionic step energy** | Yes | Yes | Yes |
| **Forces** | Yes (with symmetry info) | No | Yes |
| **Stress tensor** | Yes | No | Yes |
| **Eigenvalues** | Yes (at each k-point) | No | Yes |
| **DOS** | Partial (summary) | No | Yes (full) |
| **Timing (per subroutine)** | Yes (detailed) | No | Partial (per SCF step only) |
| **Memory usage** | Yes | No | No |
| **Warnings** | Yes | No | No |
| **Symmetry analysis** | Yes (detailed) | No | Minimal |
| **Parallelization info** | Yes | No | No |
| **Charge density info** | Partial (mixing params) | Partial (ncg) | No |
| **Dielectric properties** | Yes (if calculated) | No | Yes (if calculated) |
| **Born effective charges** | Yes (if calculated) | No | Yes (if calculated) |
| **Format** | Unstructured text | Structured columnar text | Structured XML |
| **Parsability** | Difficult (regex-dependent) | Easy (fixed format) | Easy (XML libraries) |
| **File size** | Large | Small | Large |

### 4.2 Unique Information per File

- **OUTCAR only:** Per-subroutine timing breakdown, memory allocation/usage, detailed warnings and diagnostics, symmetry operation details, charge mixing algorithm internals, POTCAR information echo, magnetization details per atom, orbital moment details. **[Documented]**
- **OSZICAR only:** Nothing truly unique — its content is a strict subset of OUTCAR. However, its compact format makes it the fastest way to check convergence. **[Observed]**
- **vasprun.xml only:** Properly structured XML representation of all theory-layer results. The XML structure is its unique value — the same data exists in OUTCAR but is much harder to parse reliably. Projected DOS/band data is more complete in vasprun.xml than OUTCAR for most configurations. **[Observed]**

### 4.3 Complete Trace Definition

For ATHENA's three-stage audit, the "complete trace" requires:

| Audit Stage | Required Files | Why |
|:---|:---|:---|
| **Stage 1: Implementation Audit** | OUTCAR + stdout + vasprun.xml | OUTCAR/stdout contain warnings, memory errors, MPI failures, parallelization diagnostics. vasprun.xml confirms whether the calculation wrote valid output. |
| **Stage 2: Methodological Audit** | vasprun.xml + INCAR + KPOINTS + POSCAR + POTCAR | vasprun.xml echoes all parameters. Cross-referencing against the input files detects specification errors. The methodological audit checks whether the calculation setup was capable of answering the scientific question. |
| **Stage 3: Theoretical Evaluation** | vasprun.xml + CONTCAR + EIGENVAL/DOSCAR | vasprun.xml provides energy, forces, eigenvalues. CONTCAR gives the final structure. EIGENVAL/DOSCAR give electronic structure for detailed analysis. |

**Minimum complete trace:** vasprun.xml + OUTCAR + stdout/stderr. OSZICAR is redundant if both of the former are available. **[Inferred]**

---

## 5. Failure Signaling

### 5.1 SCF Non-Convergence

**Description:** The self-consistent field electronic minimization fails to converge within NELM iterations.

**Signals [documented/observed]:**

| Signal | Location | Content |
|:---|:---|:---|
| OSZICAR | Last line of electronic block | Energy oscillation or slow dE decay; final dE > EDIFF |
| OUTCAR | Warning message | "WARNING: Sub-Space-Matrix is not hermitian" or no explicit non-convergence warning (VASP often continues silently) |
| vasprun.xml | `<scstep>` count equals NELM; energy change in last step > EDIFF | Number of `<scstep>` elements = NELM within a `<calculation>` block |
| stdout | May contain warnings | "ZBRENT: fatal error" or "EDDIAG: call to ZHEGV failed" in pathological cases |
| Exit code | Usually 0 (!) | VASP does NOT set a non-zero exit code for SCF non-convergence in most cases **[Observed]** |

**Fault classification:** **Ambiguous — methodology or theory.** SCF non-convergence can result from:
- Insufficient NELM (methodology: increase iteration count)
- Poor initial charge density or wavefunction (methodology: different ICHARG/ISTART)
- Inappropriate ALGO for the system (methodology: switch algorithm)
- Physically pathological electronic structure (theory: strongly correlated system requiring DFT+U or hybrid functional)
- Numerical instability from LREAL or PREC settings (implementation/methodology boundary)
- Genuine failure of the chosen exchange-correlation functional to describe the system (theory)

**ATHENA implication:** SCF non-convergence is one of the most common VASP failures and one of the hardest to classify. It sits at the methodology-theory boundary. The Trace Semantics Engine must distinguish between "the SCF algorithm failed to find the minimum" (methodology) and "the chosen functional cannot describe this system" (theory). This distinction often requires domain knowledge about the system (e.g., is it a Mott insulator? a metallic system treated with too small a smearing?). **[Inferred]**

### 5.2 Ionic Relaxation Failure

**Description:** Ionic positions fail to converge within NSW steps, or forces remain above EDIFFG threshold.

**Signals [documented/observed]:**

| Signal | Location | Content |
|:---|:---|:---|
| OSZICAR | All NSW ionic steps present with non-decreasing energy or oscillating forces | Count of ionic step blocks = NSW |
| OUTCAR | Forces summary at final step | Maximum force component > |EDIFFG| |
| vasprun.xml | Number of `<calculation>` blocks = NSW; final forces > threshold | Programmatically checkable |
| CONTCAR | Structure written at last ionic step | May not represent a minimum |
| Exit code | Usually 0 | VASP does NOT set a non-zero exit code for ionic non-convergence **[Observed]** |

**Fault classification:** **Methodology or theory.**
- Insufficient NSW (methodology: increase step count)
- Inappropriate IBRION for the energy surface (methodology: switch from CG to quasi-Newton or vice versa)
- Too aggressive EDIFFG (methodology: relax criterion)
- Structure is at a saddle point or the potential energy surface is pathological for the chosen functional (theory)
- Cell constraints (ISIF) preventing the true minimum (methodology: wrong degrees of freedom)

### 5.3 Memory Errors and Segfaults

**Description:** VASP crashes due to insufficient memory, stack overflow, or segmentation fault.

**Signals [documented/observed]:**

| Signal | Location | Content |
|:---|:---|:---|
| stdout/stderr | Error message | "forrtl: severe (174): SIGSEGV" or "insufficient virtual memory" or "SBRK: increase heap size" or MPI-related crash messages |
| OUTCAR | May be truncated | If crash occurs mid-write, OUTCAR may be incomplete |
| vasprun.xml | May be truncated/invalid | XML may be malformed (unclosed tags) if crash occurs during write |
| Exit code | Non-zero | Typically 134 (SIGABRT) or 139 (SIGSEGV) or MPI error code **[Observed]** |
| OSZICAR | May be truncated | Last line may be incomplete |

**Fault classification:** **Implementation.** Memory errors are almost always implementation-layer failures caused by:
- NCORE/KPAR settings incompatible with available memory
- ENCUT too high for available memory (though ENCUT is a theory parameter, the failure is implementation)
- System too large for available resources
- VASP binary compiled without sufficient stack size

**Caveat:** In rare cases, memory errors can mask theory-layer issues — a system that legitimately requires a very large ENCUT due to hard pseudopotentials is a case where the theory requirement (high ENCUT) creates an implementation constraint (large memory). **[Inferred]**

### 5.4 Basis Set Issues (ENCUT Too Low)

**Description:** The plane-wave cutoff energy is insufficient for the pseudopotentials being used, leading to inaccurate results.

**Signals [documented/observed]:**

| Signal | Location | Content |
|:---|:---|:---|
| OUTCAR | Warning (VASP 6+) | "WARNING: ENCUT is below the recommended value of XXX" — VASP compares ENCUT against ENMAX in POTCAR **[Documented]** |
| vasprun.xml | `<incar>` + `<atominfo>` | ENCUT value can be cross-referenced against POTCAR ENMAX values present in atominfo |
| Results | Energy, forces, stress | Pulay stress in stress tensor; total energy not converged with respect to ENCUT |
| Exit code | 0 | VASP does not fail for low ENCUT; it produces inaccurate results silently **[Observed]** |

**Fault classification:** **Theory.** ENCUT is a theory parameter — it determines the completeness of the basis set. An insufficient ENCUT means the quantum mechanical calculation is not converged with respect to the basis, and the results are physically unreliable. However, the failure is silent (no crash, no error), making it detectable only through convergence testing or cross-referencing against POTCAR ENMAX values.

**ATHENA implication:** This is a critical failure mode because it is invisible in the trace. The Trace Semantics Engine would need to implement a POTCAR-ENCUT cross-check as a rule-based validation, not rely on VASP's own signaling. **[Inferred]**

### 5.5 K-Point Sampling Inadequacy

**Description:** The Brillouin zone sampling mesh is too coarse, leading to inaccurate energies, forces, and electronic structure.

**Signals [documented/observed]:**

| Signal | Location | Content |
|:---|:---|:---|
| OUTCAR/vasprun.xml | IBZKPT content or `<kpoints>` node | Number of irreducible k-points |
| Results | Energy, DOS | Noisy or unphysical DOS; energy not converged with k-point density |
| OUTCAR | Warning (rare) | VASP rarely warns about insufficient k-points **[Observed]** |
| Exit code | 0 | No failure signal **[Observed]** |

**Fault classification:** **Theory.** K-point sampling determines the accuracy of Brillouin zone integration. Insufficient sampling is a theory-layer error (the calculation does not adequately represent the physics). Like ENCUT, this is a silent failure — detectable only through convergence tests or domain heuristics (e.g., metals require denser k-meshes than insulators).

**ATHENA implication:** Another invisible failure. The Trace Semantics Engine would need domain-aware heuristics: is the system metallic? What is the unit cell size? Is the k-mesh density (k-points per reciprocal Angstrom) within accepted norms for the system type? This requires the Causal Graph Manager to encode domain knowledge about k-point requirements. **[Inferred]**

### 5.6 Failure Classification Summary

| Failure Mode | Primary Signal Source | Explicit Signal? | Exit Code? | ATHENA Fault Class | Confidence |
|:---|:---|:---|:---|:---|:---|
| SCF non-convergence | OSZICAR, vasprun.xml | Implicit (must infer from iteration count) | No | Ambiguous (methodology/theory) | Medium |
| Ionic non-convergence | OSZICAR, vasprun.xml, OUTCAR | Implicit (must infer from step count + forces) | No | Methodology or theory | Medium |
| Memory crash | stdout/stderr, exit code | Explicit (crash) | Yes | Implementation | High |
| Segfault | stdout/stderr, exit code | Explicit (crash) | Yes | Implementation | High |
| MPI error | stdout/stderr, exit code | Explicit (crash) | Yes | Implementation | High |
| ENCUT too low | OUTCAR warning (VASP 6+), cross-check | Partially explicit | No | Theory | Low (often silent) |
| K-point inadequacy | No direct signal | No | No | Theory | Very low (always silent) |
| Wrong pseudopotential | No direct signal | No | No | Theory | Very low (always silent) |
| PREC-induced error | No direct signal | No | No | Ambiguous | Very low (always silent) |
| Symmetry error | OUTCAR warnings | Sometimes explicit | No | Implementation or methodology | Medium |

---

## 6. DFT-Specific Theory-Implementation Distinction

### 6.1 Mapping DFT to ATHENA's Three-Stage Audit

DFT calculations have a specific structure that maps onto ATHENA's fault isolation framework. The key insight is that DFT's "theory" is the exchange-correlation functional and associated approximations, NOT the Kohn-Sham equations themselves (which are exact within the DFT framework — the approximation is in the functional).

| DFT Concept | ATHENA Layer | Rationale |
|:---|:---|:---|
| Exchange-correlation functional (GGA, meta-GGA, hybrid) | **Theory (hard core)** | This is the fundamental approximation. If a PBE calculation fails to reproduce experimental lattice parameters, it may be because PBE systematically overestimates lattice constants — a theory failure. |
| Pseudopotential / PAW dataset | **Theory (hard core)** | The frozen-core approximation and the PAW reconstruction determine which physics is captured. A bad POTCAR choice can miss semi-core state effects. |
| Basis set (ENCUT) | **Theory with implementation consequences** | In principle, ENCUT should be converged to the point where it does not matter. In practice, it is a theory parameter because incomplete basis sets change the physics. But choosing ENCUT also determines memory/time requirements. |
| K-point sampling | **Theory with implementation consequences** | Same logic as ENCUT: should be converged, but practically is a theory parameter. Insufficient k-points change the physical result. |
| SCF algorithm (ALGO, NELM) | **Implementation (protective belt)** | The algorithm used to solve the Kohn-Sham equations should not affect the converged result. If it does, the system is pathological, and the failure is really about the functional's behavior on this system. |
| Ionic relaxation algorithm (IBRION) | **Methodology (protective belt)** | The optimizer should find the same minimum regardless of algorithm. If it does not, the potential energy surface may have multiple minima — a methodology issue (wrong starting structure) or theory issue (functional gives wrong surface). |
| Smearing (ISMEAR, SIGMA) | **Theory/methodology boundary** | For metals, Methfessel-Paxton smearing with small SIGMA is methodology. For semiconductors, tetrahedron method is the correct theory. Using the wrong smearing for the system type is a methodology error. |
| Parallelization (NCORE, KPAR) | **Implementation** | Pure execution parameters. Should not affect results. If they do, it is a VASP bug (implementation failure at the framework level). |
| DFT+U parameters (U, J values) | **Theory (hard core)** | The Hubbard U value is a theory parameter that fundamentally changes the physics. Different U values can give different ground states. |

### 6.2 The Three-Stage Audit Applied to VASP

**Stage 1 (Implementation Audit) for VASP:**
- Did VASP complete without crashes, segfaults, or MPI errors? (Check exit code, stdout/stderr)
- Are output files complete and well-formed? (Check vasprun.xml validity, OUTCAR completeness)
- Are there framework-level warnings indicating numerical problems? (Check OUTCAR for ZBRENT errors, sub-space matrix warnings, FFT grid issues)
- Did parallelization execute correctly? (Check OUTCAR timing section, stdout for MPI messages)
- Is the calculation reproducible with different NCORE/KPAR settings? (This is an additional probe, not directly from trace)

**Stage 2 (Methodological Audit) for VASP:**
- Is ENCUT converged for this system? (Cross-check against POTCAR ENMAX; ideally, compare with a higher-ENCUT calculation)
- Is the k-point mesh adequate for this system type (metal vs. insulator vs. semiconductor)? (Heuristic check against lattice parameters)
- Is ISMEAR appropriate for the system? (Tetrahedron for insulators, Methfessel-Paxton for metals)
- Is SIGMA small enough that the entropy contribution (TOTEN - TOTEN_free) is negligible? (Check vasprun.xml energy breakdown)
- For relaxation: Is EDIFFG tight enough? Is NSW sufficient? Is IBRION appropriate?
- For DFT+U: Is the formalism (LDAUTYPE) standard for this application?
- Is the cell size sufficient (no artificial periodicity interactions)?

**Stage 3 (Theoretical Evaluation) for VASP:**
- Do the results (lattice parameters, band gap, formation energy, magnetic moment) agree with or contradict the hypothesis?
- If the functional predicts a metallic state but the system is known to be an insulator, is this a theory failure (GGA band gap problem) or a methodology failure (too few k-points)?
- Are the forces and stress converged, and do they indicate the expected structural behavior?
- Does the electronic structure (DOS, band structure) show the expected features?

### 6.3 Key Difficulty: Silent Theory Failures

The most challenging aspect of VASP trace analysis for ATHENA is that the most important theory failures are silent. VASP will happily produce results with:
- An unconverged basis set (low ENCUT) — no error, no warning (VASP 5), mild warning (VASP 6)
- Insufficient k-points — no error, no warning
- An inappropriate functional for the system (e.g., GGA for a Mott insulator) — no error, no warning
- Wrong pseudopotential choice — no error, no warning

These silent failures require the Trace Semantics Engine to go beyond parsing VASP output and implement domain-aware validation rules. This is a significant difference from crash-based failures (which are self-announcing) and from MD codes (where force field mismatches often produce immediate numerical instabilities). **[Inferred]**

---

## 7. Closed-Source Constraints

### 7.1 What VASP's Proprietary Nature Means

VASP is proprietary commercial software distributed under academic and commercial licenses. Source code is available to licensees but cannot be redistributed, modified for public distribution, or openly instrumented. **[Documented]**

**Impact on ATHENA:**

| Constraint | Impact | Severity |
|:---|:---|:---|
| **No custom instrumentation** | Cannot add logging, tracing, or hooks into VASP internals. ATHENA is limited to output files and stdout/stderr. | **High** |
| **No access to internal state** | Cannot observe intermediate wavefunctions, charge mixing internals, or Davidson iteration details beyond what VASP chooses to write. | **High** |
| **Version-dependent behavior** | Internal algorithm changes between versions (5.x, 6.x) can change behavior for the same input. No public changelog for all internal changes. | **Medium** |
| **POTCAR distribution restrictions** | POTCAR files are part of the VASP distribution and cannot be freely redistributed. This affects test case reproducibility. | **Medium** |
| **Binary output formats** | WAVECAR and CHG formats are documented only via community reverse-engineering and VASP source. No official specification. | **Medium** |
| **No API beyond file I/O** | VASP has no programmatic API — interaction is entirely through input files and output files. No library mode, no Python bindings, no socket interface (unlike some newer codes). | **High** |

### 7.2 Comparison with Open-Source Alternatives

| Feature | VASP (proprietary) | Quantum ESPRESSO (open-source) | GPAW (open-source) |
|:---|:---|:---|:---|
| Source access | License-restricted | GPL | GPL |
| Custom instrumentation | Not possible without license | Freely modifiable | Freely modifiable |
| Output completeness | Good (vasprun.xml is comprehensive) | Good (XML output available) | Good (Python-native, full state accessible) |
| Python API | None (file I/O only) | Limited (some Python wrappers) | Native Python + C extensions |
| Internal state access | Output files only | Output files + source modification | Full Python access to all internal objects |
| Community tooling | Excellent (pymatgen, ASE, custodian) | Good (QE tools, ASE) | Good (ASE, native Python) |

### 7.3 Mitigation Strategies

Despite closed-source constraints, VASP's output system is well-designed for external analysis:

1. **vasprun.xml is comprehensive.** It captures nearly all theory-layer results in a structured format. This was a deliberate design choice by the VASP developers. **[Documented]**

2. **pymatgen's `custodian` package** provides automated error detection and correction for VASP runs. It parses OUTCAR and stdout for known error patterns and applies fixes. This is effectively a community-built implementation audit tool that ATHENA could leverage or learn from. **[Documented]**

3. **ASE's VASP calculator** provides a Python interface to VASP that handles input generation and output parsing. While it does not give access to VASP internals, it normalizes the input/output interface. **[Documented]**

4. **OUTCAR parsing patterns are well-established.** The pymatgen `Outcar` class handles the most common parsing tasks. Regex patterns for VASP warnings and errors are cataloged in custodian's error handlers. **[Documented]**

### 7.4 Assessment for ATHENA

**Can ATHENA operate effectively with VASP given closed-source constraints?**

**Yes, with caveats.** The combination of vasprun.xml (structured theory-layer results) + OUTCAR (implementation diagnostics, warnings) + stdout/stderr (crash information) + exit codes provides sufficient information for:
- Stage 1 (Implementation Audit): Detect crashes, memory errors, MPI failures, framework warnings.
- Stage 2 (Methodological Audit): Verify parameter adequacy through cross-checks (ENCUT vs. POTCAR, k-mesh density vs. system type, ISMEAR vs. electronic character).
- Stage 3 (Theoretical Evaluation): Compare results (energy, structure, electronic properties) against hypothesis predictions.

**The gap** is in Stage 1 depth: ATHENA cannot detect subtle numerical issues that VASP does not report (e.g., FFT aliasing at the margin, subtle PAW reconstruction errors, non-deterministic MPI reductions). These are rare but real failure modes that are invisible without source access.

**The risk** is proportional to how often real VASP failures fall into the "invisible" category. For most standard DFT calculations, the observable output is sufficient. For edge cases (heavy elements with strong SOC, metastable magnetic states, strongly correlated systems), the invisible failure rate increases. **[Inferred]**

**Recommendation:** VASP should remain in ATHENA's target DSL set, but with a documented limitation: ATHENA's implementation audit for VASP is bounded by VASP's own reporting completeness. For maximum coverage, ATHENA should also support at least one open-source DFT code (Quantum ESPRESSO or GPAW) where deeper instrumentation is possible, allowing cross-validation of results obtained with VASP. **[Inferred]**

---

## 8. Summary and Implications for ATHENA IR Design

### 8.1 Key Findings

1. **VASP's output system is well-structured for theory-layer reconstruction.** vasprun.xml provides a comprehensive, machine-readable record of inputs, parameters, and results. This is better than most simulation codes.

2. **The theory-implementation boundary exists but is not API-declared.** ATHENA must maintain its own classification of INCAR tags, POTCAR variants, and KPOINTS settings into theory/implementation/ambiguous categories. This is a finite engineering task.

3. **Theory is distributed across four input files.** Unlike MD codes where the force field is a single external file, VASP distributes theory across INCAR (functional, convergence), POSCAR (structure), POTCAR (pseudopotentials), and KPOINTS (BZ sampling). The IR must capture all four.

4. **The most dangerous failures are silent.** VASP does not signal insufficient ENCUT, inadequate k-points, or inappropriate functional choice. The Trace Semantics Engine must implement domain-aware validation rules beyond what VASP reports.

5. **Closed-source constraints are manageable but impose a ceiling.** ATHENA cannot instrument VASP internals, but the observable output is sufficient for most fault isolation tasks. The ceiling is hit for subtle numerical issues and non-deterministic parallelism effects.

6. **Community tooling is excellent.** pymatgen, ASE, and custodian provide mature parsing and error-handling infrastructure that ATHENA can build on rather than replace.

### 8.2 Implications for IR Schema Design

The VASP trace analysis suggests the following IR requirements:

1. **Multi-file trace composition.** The IR must be able to ingest and fuse data from multiple output files (vasprun.xml + OUTCAR + stdout) into a single semantic representation. This is different from single-log parsing.

2. **Declarative input reconstruction.** The IR must reconstruct the complete calculation specification from vasprun.xml's parameter echo, enabling cross-checks (e.g., ENCUT vs. POTCAR ENMAX).

3. **Convergence trajectory representation.** The IR must represent SCF and ionic convergence as trajectories (sequences of energy/force values), not just final states. The trajectory shape contains diagnostic information (oscillation = poor mixing, monotone decrease = healthy convergence, plateau = near convergence).

4. **Silent failure detection rules.** The IR must support rule-based validation layers that check for domain-specific adequacy conditions (ENCUT convergence, k-mesh density, smearing appropriateness) that VASP does not enforce.

5. **Ambiguous parameter annotation.** The IR must tag parameters like PREC, LREAL, and ALGO as "ambiguous layer" rather than forcing them into theory or implementation, enabling the LFI to handle them with appropriate caution.

6. **Theory-distributed-across-files representation.** The IR must have a unified representation for the full calculation specification that draws from INCAR + POSCAR + POTCAR + KPOINTS, not just INCAR.

### 8.3 Comparison with MD Codes (Preliminary)

This analysis suggests VASP differs from MD codes in several ways relevant to IR design:

| Aspect | VASP (DFT) | MD Codes (OpenMM, GROMACS) |
|:---|:---|:---|
| Theory specification | Distributed across 4 input files | Centralized in topology + force field files |
| Theory-implementation boundary | Not API-declared; requires external classification | More cleanly separated by API design |
| Input format | Purely declarative (no scripting) | May require scripting (OpenMM) or structured input (GROMACS) |
| Primary output format | XML (vasprun.xml) + text (OUTCAR) | Binary trajectories + text logs |
| Silent failure prevalence | High (ENCUT, k-points, functional choice) | Medium (force field applicability, timestep adequacy) |
| Community parsing tools | Excellent (pymatgen, ASE) | Excellent (MDAnalysis, MDTraj) |
| Closed-source constraint | Yes (VASP) | No (OpenMM, GROMACS) |

These differences suggest the IR cannot be a simple one-size-fits-all schema — it needs DSL-specific adapters that normalize diverse input/output patterns into a common semantic representation. The adapter for VASP would handle multi-file fusion and silent failure detection; the adapter for OpenMM would handle Python script parsing and trajectory analysis.

---

## References

- VASP Wiki: https://www.vasp.at/wiki/ — Primary documentation for all INCAR tags, output files, and calculation types. **[Documented source]**
- pymatgen documentation: https://pymatgen.org/ — Python Materials Genomics library; provides `Vasprun`, `Outcar`, `Oszicar`, `Chgcar`, `Eigenval`, `Doscar`, `Procar` parsers. **[Documented source]**
- pymatgen custodian: https://materialsproject.github.io/custodian/ — Automated error handling for VASP; catalogs common errors and fixes. **[Documented source]**
- ASE VASP calculator: https://wiki.fysik.dtu.dk/ase/ase/calculators/vasp.html — Atomic Simulation Environment VASP interface. **[Documented source]**
- Materials Project documentation: https://docs.materialsproject.org/ — Large-scale VASP workflow patterns. **[Documented source]**
- VASP forum: https://www.vasp.at/forum/ — Community discussions of errors and failure patterns. **[Community source]**
