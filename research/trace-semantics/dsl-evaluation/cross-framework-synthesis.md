# Cross-Framework Trace Synthesis

**Investigation:** Trace Semantics Engine — DSL Trace Format Cross-Framework Synthesis (Step 1d)
**Date:** 2026-02-20
**Scope:** Comparative analysis of OpenMM, GROMACS, and VASP trace output systems
**Input Documents:** openmm-trace-analysis.md, gromacs-trace-analysis.md, vasp-trace-analysis.md
**Architecture References:** ARCHITECTURE.md §4.5, §5.3; VISION.md §4.1

---

## 1. Trace Capability Matrix

The following table compares trace element categories across all three frameworks. Each cell contains: the data format, the access method (native = default output, custom = requires additional configuration or external tooling, unavailable = not producible by the framework), and the layer classification (theory / implementation / boundary / mixed).

### 1.1 State Snapshots (Coordinates, Velocities, Forces)

| Element | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|
| **Coordinates** | DCD binary (DCDReporter, native), PDB text (PDBReporter, native), PDBx/mmCIF text (PDBxReporter, native), XTC binary (mdtraj XTCReporter, custom), HDF5 (mdtraj HDF5Reporter, custom). Layer: theory. [OpenMM §1.2-1.7] | .trr binary full-precision (nstxout, native), .xtc binary lossy-compressed (nstxout-compressed, native), .gro text final frame (native). Layer: theory. [GROMACS §1.4-1.7] | CONTCAR text POSCAR format final frame (native), XDATCAR text multi-frame POSCAR all ionic steps (native), vasprun.xml per-ionic-step `<structure>` (native). Layer: theory. [VASP §1.1, §1.3] |
| **Velocities** | Via custom reporter calling `state.getVelocities()` (custom), HDF5Reporter optional (custom). Not emitted by any core reporter by default. Layer: theory. [OpenMM §1.7, §5.2] | .trr binary (nstvout, native but often disabled). Layer: theory. [GROMACS §1.4] | Not output by VASP. Velocities are internal to the SCF solver and not part of the DFT output semantics. For AIMD, velocities are in XDATCAR implicitly (finite differences) but not directly reported. Layer: not applicable for static DFT; theory for AIMD. [VASP §1.3] |
| **Forces** | Via custom reporter calling `state.getForces()` (custom), HDF5Reporter optional (custom). Not emitted by any core reporter by default. Layer: theory. [OpenMM §5.2] | .trr binary (nstfout, native but often disabled). Layer: theory. [GROMACS §1.4] | OUTCAR text per-ionic-step (native), vasprun.xml `<varray name="forces">` per-ionic-step (native). Layer: theory. [VASP §1.1, §3.2] |

### 1.2 Energy Series (Total, Decomposed, Per-Term)

| Element | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|
| **Total energy** | StateDataReporter CSV `totalEnergy` column (native). Layer: theory. [OpenMM §1.1] | .edr binary time series via `panedr` (native). Layer: theory. [GROMACS §1.3, §4.1] | vasprun.xml `<energy>` per ionic step and per SCF step (native), OSZICAR text per ionic step (native), OUTCAR text (native). Layer: theory. [VASP §1.1, §3.1] |
| **Kinetic energy** | StateDataReporter CSV `kineticEnergy` column (native). Layer: theory. [OpenMM §1.1] | .edr binary time series (native). Layer: theory. [GROMACS §4.1] | Not directly output (DFT electronic kinetic energy is part of total energy decomposition in vasprun.xml `<scstep>/<energy>`, but not as a separate thermodynamic observable). Layer: theory (embedded in energy decomposition). [VASP §3.1] |
| **Potential energy** | StateDataReporter CSV `potentialEnergy` column (native). Layer: theory. [OpenMM §1.1] | .edr binary time series (native). Layer: theory. [GROMACS §4.1] | vasprun.xml `<energy>` components per ionic step (native). Layer: theory. [VASP §3.1] |
| **Per-term decomposition** | Not available from default reporters. Requires custom reporter calling `context.getState(getEnergy=True, groups={i})` for each force group (custom). Layer: theory. [OpenMM §5.2, §5.3] | .edr binary includes per-force-type terms: Bond, Angle, Dihedral, LJ-SR, Coul-SR, Coul-recip, Dispersion correction, plus per-group decomposition if `energygrps` configured (native). Layer: theory. [GROMACS §4.1] | vasprun.xml `<scstep>/<energy>` includes per-SCF-step decomposition: alphaZ, ewald, hartreedc, XC, and other DFT energy components (native). Layer: theory. [VASP §3.1, §3.2] |

### 1.3 Convergence Metrics

| Element | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|
| **SCF iterations** | Not applicable (MD codes do not perform SCF). | Not applicable. | OSZICAR per-SCF-step energy/dE/d eps/ncg (native), vasprun.xml `<scstep>` count and energy per step (native), OUTCAR detailed SCF convergence (native). Layer: mixed (convergence trajectory is theory-relevant; iteration count is implementation). [VASP §1.1, §3.1, §5.1] |
| **Constraint violations** | Exception thrown on constraint failure ("Constraint tolerance not met"), but no per-step constraint metrics reported by default (unavailable without custom reporter). Layer: methodology/implementation boundary. [OpenMM §3.1] | LINCS warnings in .log with rms/max deviation and bond rotation angles (native). Constraint energy term in .edr (native). Layer: methodology. [GROMACS §5.2] | Not applicable (DFT does not use bond constraints). |
| **Energy drift** | Not computed by OpenMM. Must be derived externally from StateDataReporter total energy time series (custom post-processing). Layer: methodology. [OpenMM §1.1] | Energy drift reported in .log final statistics section (native). Also derivable from .edr time series (native + custom post-processing). Layer: methodology. [GROMACS §3.2] | Not applicable in the MD sense. SCF energy convergence (dE per SCF step) serves an analogous role; available in OSZICAR and vasprun.xml (native). Layer: theory/methodology. [VASP §5.1] |

### 1.4 Error/Warning Messages

| Element | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|
| **Typed exceptions** | Python `OpenMMException` with free-text message. No structured error codes. Covers: template matching failures, CUDA errors, NaN coordinates, constraint failures. Layer: mixed. [OpenMM §3.1] | Free-text messages in .log and stderr. Three severity levels (NOTE, WARNING, Fatal error) but no numeric error codes. Covers: LINCS/SETTLE/SHAKE, domain decomposition, NaN forces, memory, MPI. Layer: mixed. [GROMACS §5.1] | Free-text in stdout/stderr and OUTCAR. No structured error codes. Covers: ZBRENT failures, sub-space matrix warnings, segfaults, MPI errors. VASP exit code is often 0 even for non-convergence. Layer: mixed. [VASP §5.1-5.5] |
| **Free-text warnings** | Not systematically emitted during simulation. Exceptions are the primary mechanism; warnings are not a separate category. Layer: N/A. [OpenMM §3.1] | Interspersed throughout .log. WARNING and NOTE markers but no machine-readable classification. Also emitted by grompp during preprocessing. Layer: mixed. [GROMACS §5.1, §3.2] | In OUTCAR (detailed) and stdout (brief). Includes: "Sub-Space-Matrix is not hermitian", ENCUT below recommended value (VASP 6+), symmetry warnings. Not in vasprun.xml. Layer: mixed. [VASP §5.1, §5.4] |
| **Silent failures** | Methodology failures (insufficient sampling, inadequate equilibration, wrong ensemble) are entirely invisible. NaN can propagate without exception if no reporter checks for it. [OpenMM §3.1, §3.3] | Methodology failures (inadequate sampling, insufficient equilibration) are invisible. Some theory failures (wrong force field for chemistry) run without error. [GROMACS §5.8, §7.2] | Theory failures (insufficient ENCUT, inadequate k-points, inappropriate functional) are silent -- VASP exits with code 0 and produces results without warning. SCF non-convergence often exits with code 0. [VASP §5.4-5.6, §6.3] |

### 1.5 Parameter Echo (Input Specification Reconstruction)

| Element | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|
| **Input parameters** | No built-in parameter echo. The System, Integrator, and Platform objects are programmatically queryable (custom -- requires API calls at runtime). Force field XML source files are not echoed. Layer: mixed. [OpenMM §2.1-2.2, §4.1] | Complete .mdp parameter echo in .log file (native), `key = value` format. All parameters including defaults. Also recoverable from .tpr via `gmx dump`. Layer: mixed (theory + implementation merged). [GROMACS §3.2, §2.1] | Complete INCAR parameter echo in vasprun.xml `<incar>` and `<parameters>` nodes (native), including resolved defaults. Also echoed in OUTCAR header (native). Layer: mixed. [VASP §3.1, §3.2] |
| **Topology/structure echo** | Topology queryable from `simulation.topology` at runtime (custom). Not written to output files by default. [OpenMM §5.2] | Complete topology and coordinates in .tpr binary (native). Recoverable via `gmx dump`. [GROMACS §1.1] | POSCAR echoed in vasprun.xml `<structure name="initialpos">` (native). POTCAR identity partially in `<atominfo>` (native). KPOINTS in `<kpoints>` (native). [VASP §3.1, §3.2] |

### 1.6 Execution Metadata (Platform, Timing, Parallelization)

| Element | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|
| **Platform/hardware** | Platform name, precision, device index, compiler queryable via `context.getPlatform()` API (custom -- requires runtime API calls). Not written to output files by default reporters. Layer: implementation. [OpenMM §4.1] | .log header: GROMACS version, build config, CPU/GPU detection, SIMD, MPI/thread counts (native). Layer: implementation. [GROMACS §3.2] | vasprun.xml `<generator>` node: VASP version, date, platform (native). OUTCAR and stdout: detailed parallelization, MPI rank info, memory allocation (native). Layer: implementation. [VASP §1.1, §1.3, §3.2] |
| **Timing** | StateDataReporter `speed` (ns/day), `elapsedTime`, `remainingTime` columns (native). No per-component timing. Layer: implementation. [OpenMM §1.1] | .log performance summary: wall time, ns/day, per-step breakdown (forces, PME, constraints, communication, neighbor search) (native). Layer: implementation. [GROMACS §3.2] | OUTCAR per-subroutine timing breakdown (native). vasprun.xml `<time>` per SCF step (native). stdout: total timing and memory (native). Layer: implementation. [VASP §1.1, §3.2] |
| **Parallelization** | Thread count (CPU), GPU device index queryable via Platform properties (custom). No parallelization efficiency metrics. Layer: implementation. [OpenMM §4.1, §4.2] | .log domain decomposition layout, PME rank assignment, load imbalance metrics (native). Layer: implementation. [GROMACS §3.2] | OUTCAR and stdout: NCORE, KPAR, NPAR settings, MPI decomposition (native). Layer: implementation. [VASP §1.3, §2.2] |

### 1.7 Trajectory Data

| Element | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|
| **Full trajectory** | DCDReporter binary (native), PDBReporter text (native), XTCReporter binary via mdtraj (custom), HDF5Reporter via mdtraj (custom). Coordinates at configurable intervals. Layer: theory. [OpenMM §1.2-1.7] | .trr full-precision binary (native), .xtc compressed binary (native). Coordinates/velocities/forces at configurable intervals. Layer: theory. [GROMACS §1.4, §1.5] | XDATCAR text multi-frame POSCAR format, one frame per ionic step (native). No sub-ionic-step trajectory. For AIMD: XDATCAR is the trajectory. For relaxation: XDATCAR records the optimization path. Layer: theory. [VASP §1.3] |
| **Checkpoints** | CheckpointReporter binary blob, platform-specific and version-specific, NOT portable (native). Alternatively, portable state XML via `writeState` option (native). Layer: implementation (binary) or mixed (XML). [OpenMM §1.5] | .cpt binary, portable across architectures, complete state including RNG and integrator state (native). Two checkpoints kept (current + previous). Layer: mixed. [GROMACS §1.6] | WAVECAR binary Fortran unformatted (native) and CHGCAR volumetric text (native) serve as restart files. Not portable across VASP versions. No single checkpoint file -- restart requires WAVECAR + CHGCAR + CONTCAR. Layer: mixed. [VASP §1.2, §1.3] |
| **Restart files** | Checkpoint binary or state XML (native). Layer: implementation/mixed. [OpenMM §1.5] | .cpt checkpoint file (native). Layer: mixed. [GROMACS §1.6] | WAVECAR (wavefunctions), CHGCAR (charge density), CONTCAR (structure). All three needed for full restart. ISTART and ICHARG INCAR tags control restart behavior. Layer: mixed. [VASP §1.2, §2.2] |

---

## 2. Theory-Implementation Boundary Assessment

### 2.1 OpenMM: CLEAN

**Rating: Clean -- API-enforced separation at `ForceField.createSystem()`.**

The theory-implementation boundary in OpenMM is enforced through a well-designed class hierarchy that physically separates theory-layer objects from implementation-layer objects in the API [OpenMM §2].

**Theory-side entities:**
- `ForceField`: Encodes the potential energy function -- force terms, functional forms, parameters. Loaded from XML files (e.g., `amber14-all.xml`). [OpenMM §2.1]
- `Topology`: Molecular structure -- atoms, bonds, residues, chains, box dimensions. [OpenMM §2.1]
- `System`: Compiled physics specification -- particle masses, Force objects with parameters, constraints. The product of `ForceField.createSystem(topology)`. [OpenMM §2.1]
- Integrator choices (`LangevinMiddleIntegrator`, `VerletIntegrator`): Equations of motion, temperature, friction, time step. [OpenMM §2.1]
- `MonteCarloBarostat`: Pressure control (target pressure, coupling frequency). [OpenMM §2.1]

**Implementation-side entities:**
- `Platform`: Compute backend (Reference, CPU, CUDA, OpenCL). [OpenMM §2.2]
- Platform properties: `Precision` (single/mixed/double), `DeviceIndex`, `CudaCompiler`, `DeterministicForces`. [OpenMM §2.2]
- `Context`: Execution container binding System + Integrator + Platform. Holds current state on the compute device. [OpenMM §2.2]
- Parallelization: Thread count (CPU), GPU device selection, multi-GPU via `CudaDeviceIndex`. [OpenMM §2.2]

**Boundary object:** `ForceField.createSystem()` is the compilation step. It takes theory-layer inputs (ForceField + Topology) and produces a System object. Several parameters at this boundary straddle layers (`nonbondedMethod`, `constraints`, `ewaldErrorTolerance`, `hydrogenMass`), but these are a small, well-defined set [OpenMM §2.3].

**Clean separation evidence:**
- `Context.getState()` provides a unified query API for physical state (positions, velocities, forces, energies) regardless of Platform [OpenMM §2.4].
- Forces are individually queryable and enumerable from the System object [OpenMM §2.4].
- A user cannot accidentally mix force field specification with GPU kernel configuration [OpenMM §2.4].

**Residual weaknesses:**
- Atom type assignment trail is lost at `createSystem()` -- the System does not retain original force field atom type labels [OpenMM §2.3].
- Platform-level numerical differences (CUDA single vs. Reference double) are real but not surfaced in output [OpenMM §2.4].
- `DeterministicForces` defaults to False on CUDA, introducing non-determinism invisible in the API [OpenMM §2.4].

### 2.2 GROMACS: SEMI-CLEAN

**Rating: Semi-clean -- .mdp parameter separation with boundary parameters that straddle theory/implementation.**

GROMACS separates theory from implementation through distinct configuration surfaces: the .mdp file (theory + some implementation parameters), the topology files (.top/.itp, purely theory), and the mdrun command line (purely implementation) [GROMACS §2.1]. The separation is structural but not perfectly enforced -- the .mdp namespace contains both theory and implementation parameters, and the .tpr binary merges both layers into a single opaque object [GROMACS §2.2].

**Strengths:**
- mdrun command-line parameters (`-ntomp`, `-ntmpi`, `-gpu_id`, `-pin`, `-dd`, `-npme`, `-nb`, `-bonded`, `-update`) are purely implementation-layer with no theory content whatsoever [GROMACS §2.1.2].
- Force field files (.itp) are entirely theory-layer [GROMACS §2.1.1].
- grompp provides a natural audit point -- the .tpr records exactly what was compiled [GROMACS §2.2].

**Boundary parameters (the complete set from the survey):**

| Parameter | Theory Role | Implementation Role | Dominance |
|:---|:---|:---|:---|
| `dt` | Integration accuracy; must resolve fastest motions | Computational cost (total steps = time / dt) | Theory-dominant [GROMACS §2.1.3] |
| `nsteps` | Total sampling time for scientific adequacy | Total compute cost | Methodology-dominant [GROMACS §2.1.3] |
| `rlist` | Buffer radius; if too small, misses interactions | Larger rlist = more pair computations | Implementation-dominant (auto-managed by Verlet scheme) [GROMACS §2.1.3] |
| `nstlist` | Neighbor list update frequency; too infrequent = missed interactions | More frequent = higher overhead | Implementation-dominant (auto-tuned) [GROMACS §2.1.3] |
| `pbc` | Periodic boundary conditions are a physical model choice | Affects domain decomposition and parallelization | Theory-dominant [GROMACS §2.1.3] |
| `fourierspacing` | PME grid spacing affects electrostatic accuracy | Finer grid = more accurate but slower | Balanced [GROMACS §2.1.3] |
| `lincs-order` | Higher order = more accurate constraint enforcement | More constraint solver iterations per step | Methodology-dominant [GROMACS §2.1.3] |
| `lincs-iter` | Number of LINCS iterations affects constraint accuracy | More iterations = slower | Methodology-dominant [GROMACS §2.1.3] |
| `verlet-buffer-tolerance` | Energy drift tolerance, used to auto-set rlist | Tighter tolerance = larger rlist = more computation | Balanced [GROMACS §2.1.3] |
| `cutoff-scheme` | Verlet vs. group -- both should produce equivalent physics | Verlet enables GPU and auto-tuning | Implementation-dominant [GROMACS §2.1.3] |

**Key challenge:** The .tpr merges both layers into a single opaque binary. Recovering layer separation requires either the original .mdp file or parsing `gmx dump` output with a classification table [GROMACS §2.3]. Additionally, GROMACS performs runtime auto-tuning (PME parameters, nstlist adjustment, Verlet buffer) that silently modifies execution parameters -- the .log records these changes but the user's original intent and the runtime-adjusted values may diverge [GROMACS §2.3].

### 2.3 VASP: DIRTY

**Rating: Dirty -- Flat INCAR namespace with no API-declared theory-implementation separation.**

VASP uses a single flat-file namespace (INCAR) where theory parameters, implementation parameters, and ambiguous parameters coexist without formal classification [VASP §2]. There is no VASP-provided metadata labeling tags as "physics" vs. "execution" -- the classification must be supplied externally [VASP §2.4].

**Key ambiguous parameters:**

| Parameter | Theory Effect | Implementation Effect | Classification Difficulty |
|:---|:---|:---|:---|
| `PREC` (Accurate/Normal/Low) | Sets ENCUT default, FFT grid density, augmentation grid density -- all affect numerical accuracy of physical results | Controls memory allocation, FFT grid sizes, wrap-around errors | **High.** A single tag simultaneously configures theory precision and implementation resource allocation. Changing PREC to save memory may inadvertently change the physical result. [VASP §2.3] |
| `LREAL` (Auto/True/False) | Real-space projection introduces controlled approximation errors in forces | Real-space projection is faster for large cells | **High.** Trades physical accuracy for speed. For small cells, LREAL=.FALSE. is required for correct physics; for large cells, LREAL=Auto is acceptable. The correctness threshold depends on cell size, which is system-specific domain knowledge. [VASP §2.3] |
| `ADDGRID` | Finer integration grid improves augmentation charge accuracy | Increases memory and computation time | **Medium.** Pure accuracy-cost tradeoff, but the accuracy impact is quantifiable through convergence testing. [VASP §2.3] |
| `ALGO` (Normal/Fast/Very_Fast/All/Damped) | Different algorithms may converge to different local SCF minima for difficult systems (magnetic, strongly correlated) | Different algorithms have different computational cost and stability | **Medium for most systems, High for pathological systems.** For well-behaved systems, ALGO is purely implementation. For systems with multiple SCF minima, ALGO can determine which physical solution is found. [VASP §2.3] |
| `ENCUT` (in practice) | Basis set completeness -- fundamental physical accuracy | Higher ENCUT = more plane waves = more memory and time | **Medium.** The "converged" ENCUT value is theory; the actual ENCUT used may be a practical compromise driven by computational budget. Listed under theory in the classification, but its practical value often straddles the boundary. [VASP §2.3] |

**Why external classification is required:** VASP's INCAR tag set is approximately 200-300 total parameters, of which roughly 50-80 are commonly used [VASP §2.4]. The theory-implementation boundary exists conceptually -- domain experts routinely distinguish between "physics parameters" (GGA, ENCUT, ISPIN) and "execution parameters" (NCORE, KPAR, LPLANE) -- but this classification is domain knowledge encoded in the practitioner's head, not in VASP's API. ATHENA's DSL adapter for VASP must maintain an explicit tag-level metadata table mapping each INCAR parameter to theory/implementation/ambiguous [VASP §2.4].

**Comparison with MD codes:** MD codes decouple the "force field" (external data files) from the engine entirely. In VASP, the equivalent of the force field (the exchange-correlation functional) is selected by an INCAR tag and compiled into the binary [VASP §2.4]. Additionally, theory is distributed across four input files (INCAR, POSCAR, POTCAR, KPOINTS) rather than centralized [VASP §8.3].

### 2.4 Boundary Parameter Catalog

The following table collects all identified boundary parameters from all three frameworks.

| Parameter | Framework | Theory Effect | Implementation Effect | Classification Difficulty |
|:---|:---|:---|:---|:---|
| `nonbondedMethod` | OpenMM | Determines long-range electrostatics treatment | Affects computational approach (PME vs. cutoff) | Low (well-understood methodology choice) |
| `constraints` | OpenMM | Constraining bonds affects dynamics | Enables larger time steps | Low |
| `ewaldErrorTolerance` | OpenMM | Controls PME accuracy | Finer grid = slower | Medium |
| `hydrogenMass` | OpenMM | Hydrogen mass repartitioning changes dynamics | Enables larger time steps | Medium |
| `dt` | GROMACS | Integration accuracy | Computational cost | Low (methodology-dominant) |
| `nsteps` | GROMACS | Sampling sufficiency | Compute cost | Low (methodology-dominant) |
| `rlist` | GROMACS | Interaction correctness | Pair computation count | Low (auto-managed) |
| `nstlist` | GROMACS | Neighbor list currency | Update overhead | Low (auto-tuned) |
| `pbc` | GROMACS | Physical model choice | Parallelization strategy | Low (theory-dominant) |
| `fourierspacing` | GROMACS | Electrostatic accuracy | Computation cost | Medium |
| `lincs-order` | GROMACS | Constraint accuracy | Solver iterations | Low (methodology-dominant) |
| `lincs-iter` | GROMACS | Constraint accuracy | Solver cost | Low (methodology-dominant) |
| `verlet-buffer-tolerance` | GROMACS | Energy drift tolerance | Pair list size | Medium |
| `cutoff-scheme` | GROMACS | Physics equivalence (ideally) | GPU enablement, auto-tuning | Low (implementation-dominant) |
| `PREC` | VASP | Basis accuracy, grid density | Memory allocation, FFT sizes | **High** |
| `LREAL` | VASP | Force accuracy (approximation in real-space projection) | Speed for large cells | **High** |
| `ADDGRID` | VASP | Augmentation charge accuracy | Memory and compute cost | Medium |
| `ALGO` | VASP | SCF solution identity (for pathological systems) | SCF solver cost and stability | Medium-High (system-dependent) |
| `ENCUT` (practical) | VASP | Basis completeness | Memory and compute cost | Medium |
| `NBANDS` (practical) | VASP | Electronic state coverage | Eigenvalue solver cost | Medium |

**Pattern:** OpenMM's boundary parameters are few and well-defined. GROMACS has more boundary parameters but most are auto-managed or methodology-dominant. VASP has the most boundary parameters with the highest classification difficulty, reflecting its flat namespace design.

---

## 3. Failure Mode Taxonomy

### 3.1 OpenMM Failure Modes

| ID | Failure Mode | Category | Signal Type | Detection Method | Confidence |
|:---|:---|:---|:---|:---|:---|
| I1 | Single-precision overflow (NaN from GPU float range) | Implementation | Implicit (NaN in energy output) or explicit (CUDA kernel error) | StateDataReporter shows NaN; compare Reference vs. CUDA platform | Medium -- requires precision comparison test [OpenMM §7.1] |
| I2 | GPU memory exhaustion | Implementation | Explicit (OpenMMException at Context creation) | Exception message with memory error | High [OpenMM §7.1] |
| I3 | CUDA/OpenCL driver incompatibility | Implementation | Explicit (OpenMMException at Context creation) | Exception message referencing CUDA initialization | High [OpenMM §7.1] |
| I4 | Non-deterministic force summation artifacts | Implementation | Silent (different trajectories across runs) | Compare multiple runs or enable DeterministicForces=true | Low -- no default signal [OpenMM §7.1] |
| I5 | Checkpoint portability failures | Implementation | Explicit (exception on load) or silent (corrupted state) | Exception or result comparison across platforms | Medium [OpenMM §7.1] |
| M1 | Insufficient energy minimization | Methodology | Silent (NaN in first ~100 steps) | NaN timing pattern (early crash) | Medium -- requires timing analysis [OpenMM §7.2] |
| M2 | Time step too large | Methodology | Implicit (gradual energy drift) | Energy drift detection in StateDataReporter time series | Low -- gradual, hard to distinguish from theory [OpenMM §7.2] |
| M3 | Inadequate equilibration | Methodology | Silent (trends in observables during "production") | Post-hoc analysis of observable time series | Low -- not detected by OpenMM [OpenMM §7.2] |
| M4 | Insufficient sampling | Methodology | Silent (unconverged statistics) | Autocorrelation analysis, block averaging | Low -- not detected by OpenMM [OpenMM §7.2] |
| M5 | Wrong ensemble choice | Methodology | Silent (incorrect thermodynamic behavior) | Cross-ensemble comparison | Low -- not detected by OpenMM [OpenMM §7.2] |
| T1 | Wrong force field for system | Theory | Implicit (incorrect properties) or explicit (createSystem exception for missing types) | Comparison against experimental data; exception at createSystem if atom types missing | Medium for exceptions, Low for silent wrong parameters [OpenMM §7.3] |
| T2 | Incorrect protonation states | Theory | Silent (wrong electrostatics) | pKa calculations, comparison with experimental structure | Low -- not detected by OpenMM [OpenMM §7.3] |
| T3 | Missing or incorrect force field parameters | Theory | Explicit (createSystem exception) or silent (wrong but present parameters) | Exception at createSystem; comparison with QM reference | Medium for missing, Low for wrong-but-present [OpenMM §7.3] |
| T4 | Missing system components (solvent, ions, cofactors) | Theory | Silent (structural collapse, incorrect dynamics) | Validation against experimental structure | Low -- not detected by OpenMM [OpenMM §7.3] |
| A1 | NaN energy without clear origin | Ambiguous | Implicit (NaN in output) | Requires per-force-group decomposition, precision comparison, time step sensitivity -- none automated | Low [OpenMM §7.4] |
| A2 | Constraint failure during dynamics | Ambiguous | Explicit (OpenMMException) | Exception message, but root cause requires cross-referencing time step, structure, and force field | Medium for detection, Low for classification [OpenMM §7.4] |
| A3 | NVE energy drift | Ambiguous | Implicit (drift in total energy) | Time step convergence test, precision comparison, PME sensitivity | Low [OpenMM §7.4] |

### 3.2 GROMACS Failure Modes

| ID | Failure Mode | Category | Signal Type | Detection Method | Confidence |
|:---|:---|:---|:---|:---|:---|
| G-I1 | Memory exhaustion | Implementation | Explicit (Fatal error: "Not enough memory") | Fatal error message in .log/stderr | High [GROMACS §5.6] |
| G-I2 | GPU error | Implementation | Explicit (Fatal error: "GPU error") | Fatal error message | High [GROMACS §5.6] |
| G-I3 | MPI failure | Implementation | Explicit (Fatal error: "MPI failed") | Fatal error message | High [GROMACS §5.6] |
| G-I4 | Atom count mismatch | Implementation | Explicit (Fatal error from grompp) | grompp validation | High [GROMACS §5.5] |
| G-I5 | Parameter incompatibility | Implementation | Explicit (Fatal error from grompp) | grompp validation | High [GROMACS §5.5] |
| G-M1 | Timestep too large | Methodology | Implicit (energy drift, LINCS warnings) | Energy drift in .edr, LINCS warnings in .log | Medium -- detected but not classified by GROMACS [GROMACS §7.2] |
| G-M2 | Insufficient equilibration | Methodology | Silent (observable drift) | Post-hoc analysis of .edr time series | Low [GROMACS §7.2] |
| G-M3 | Insufficient sampling | Methodology | Silent (unconverged statistics) | Autocorrelation analysis | Low [GROMACS §7.2] |
| G-M4 | Wrong ensemble (Berendsen thermostat) | Methodology | Explicit WARNING from grompp | grompp warning message | Medium -- warned but not prevented [GROMACS §7.1] |
| G-M5 | Periodic image artifacts | Methodology | Silent (self-interaction) | grompp checks box vs. cutoff but not molecule size vs. box | Low [GROMACS §7.2] |
| G-T1 | Incorrect force field assignment (valid but wrong types) | Theory | Silent (wrong physics) | Comparison with experimental data | Low [GROMACS §7.2] |
| G-T2 | Missing force field parameters | Theory | Explicit (grompp fatal error) | grompp validation | High [GROMACS §7.1] |
| G-T3 | Non-physical force field (syntactically valid) | Theory | Silent (non-physical results) | Comparison with experimental data | Low [GROMACS §7.2] |
| G-T4 | Subtle topology errors (wrong dihedrals, multiplicities) | Theory | Silent (incorrect conformational behavior) | Comparison with QM or experimental reference | Low [GROMACS §7.2] |
| G-A1 | LINCS/SETTLE/SHAKE constraint failure | Ambiguous | Explicit (WARNING/Fatal error in .log) | Error message identifies failing atoms; root cause requires cross-referencing .mdp, .edr, topology, initial structure, failure timing | Medium for detection, Low for classification [GROMACS §5.2] |
| G-A2 | Domain decomposition failure | Ambiguous | Explicit (Fatal error: "atom not at expected position") | Error message; usually a symptom of methodology/theory root cause | Medium for detection, Low for root cause [GROMACS §5.3] |
| G-A3 | NaN in forces | Ambiguous | Explicit (Fatal error: "non-finite values in force array") | Error message; root cause can be overlapping atoms (impl), missing LJ params (theory), or precision (impl) | Medium for detection, Low for classification [GROMACS §5.4] |

### 3.3 VASP Failure Modes

| ID | Failure Mode | Category | Signal Type | Detection Method | Confidence |
|:---|:---|:---|:---|:---|:---|
| V-I1 | Memory crash (insufficient memory) | Implementation | Explicit (segfault/SIGABRT, non-zero exit code) | Exit code + stdout/stderr error message | High [VASP §5.3] |
| V-I2 | Segmentation fault | Implementation | Explicit (SIGSEGV, non-zero exit code) | Exit code + stderr | High [VASP §5.3] |
| V-I3 | MPI error | Implementation | Explicit (MPI error code, non-zero exit code) | Exit code + stderr | High [VASP §5.3] |
| V-I4 | VASP binary/compilation issue | Implementation | Explicit (crash at startup) | Crash before any output produced | High [VASP §5.3] |
| V-M1 | Insufficient NELM (SCF iteration limit too low) | Methodology | Implicit (SCF step count = NELM, dE > EDIFF) | Count `<scstep>` elements in vasprun.xml; check final dE vs. EDIFF | Medium [VASP §5.1] |
| V-M2 | Insufficient NSW (ionic step limit too low) | Methodology | Implicit (ionic step count = NSW, forces > EDIFFG) | Count `<calculation>` blocks in vasprun.xml; check final forces | Medium [VASP §5.2] |
| V-M3 | Inappropriate ALGO for system | Methodology | Implicit (SCF oscillation or slow convergence) | SCF convergence trajectory shape analysis in OSZICAR/vasprun.xml | Medium [VASP §5.1] |
| V-M4 | Wrong ISMEAR for system type | Methodology | Silent (incorrect BZ integration) | Cross-check ISMEAR against system metallic/insulating character | Low [VASP §6.1] |
| V-M5 | SIGMA too large (excessive smearing) | Methodology | Implicit (large entropy contribution in energy) | Compare TOTEN vs. TOTEN_free in vasprun.xml energy decomposition | Medium [VASP §6.2] |
| V-T1 | ENCUT too low (incomplete basis) | Theory | Silent (unconverged results, no crash, exit code 0) or explicit WARNING (VASP 6+) | Cross-check ENCUT vs. POTCAR ENMAX; convergence testing | Low for VASP 5 (silent), Medium for VASP 6 (warning) [VASP §5.4] |
| V-T2 | K-point mesh inadequate | Theory | Silent (unconverged results, no crash, exit code 0) | Heuristic check of k-mesh density vs. cell size and system type | Low (always silent) [VASP §5.5] |
| V-T3 | Wrong pseudopotential choice | Theory | Silent (wrong valence electron treatment) | Cross-check POTCAR identity against best practices for element/system | Low (always silent) [VASP §5.6] |
| V-T4 | Inappropriate exchange-correlation functional | Theory | Silent (systematic errors in properties) | Comparison against experimental data; domain knowledge | Low (always silent) [VASP §6.3] |
| V-T5 | Missing or incorrect DFT+U parameters | Theory | Silent (wrong electronic structure for correlated systems) | Comparison against experimental band gap / magnetic properties | Low [VASP §6.1] |
| V-A1 | SCF non-convergence (ambiguous origin) | Ambiguous | Implicit (NELM steps reached, dE > EDIFF, often exit code 0) | SCF step count + energy trajectory analysis; root cause can be methodology (algorithm), theory (functional), or implementation (PREC/LREAL) | Medium for detection, Low for classification [VASP §5.1] |
| V-A2 | PREC-induced inaccuracy | Ambiguous | Silent (subtly different physics from PREC=Low vs. Accurate) | Compare results at different PREC values | Low (always silent) [VASP §5.6] |
| V-A3 | Symmetry error | Ambiguous | Sometimes explicit (OUTCAR warnings) | OUTCAR warning messages; may be methodology (wrong structure) or implementation (symmetry detection failure) | Medium [VASP §5.6] |

### 3.4 Harmonized Cross-Framework Taxonomy

The following cross-framework patterns emerge from comparing the three individual taxonomies.

#### Common failure patterns (present in all three or two of three frameworks):

| Harmonized Pattern | OpenMM | GROMACS | VASP | Category |
|:---|:---|:---|:---|:---|
| **Numerical overflow / NaN generation** | I1 (single-precision overflow to NaN), A1 (NaN ambiguous origin) | G-A3 (NaN in forces) | V-I1 (memory crash from ENCUT-driven allocation) -- not NaN per se, but analogous numerical limit | Implementation or ambiguous. Common across MD codes; VASP's equivalent manifests differently (memory rather than NaN) due to DFT's iterative solver architecture. |
| **Resource exhaustion (memory, GPU)** | I2 (GPU memory exhaustion) | G-I1 (memory exhaustion), G-I2 (GPU error) | V-I1 (memory crash), V-I2 (segfault) | Implementation. Universally explicit (exception/crash/non-zero exit). Highest-confidence classification across all frameworks. |
| **Parallelization/platform failure** | I3 (CUDA/OpenCL driver), I5 (checkpoint portability) | G-I3 (MPI failure) | V-I3 (MPI error), V-I4 (binary issue) | Implementation. Universally explicit. |
| **Constraint/solver convergence failure** | A2 (constraint failure during dynamics) | G-A1 (LINCS/SETTLE/SHAKE failure) | V-A1 (SCF non-convergence) | Ambiguous. Structurally analogous: all three involve iterative solvers failing to converge, with root causes spanning methodology (wrong parameters), theory (wrong model), and implementation (numerical precision). This is the hardest-to-classify failure pattern in all frameworks. |
| **Input validation failure** | T1/T3 (createSystem exception for missing types/parameters) | G-I4 (atom count mismatch), G-I5 (parameter incompatibility), G-T2 (missing FF parameters) | N/A (VASP has minimal input validation -- most specification errors are silent) | Implementation or theory. OpenMM and GROMACS catch these early (at compilation/preprocessing). VASP's lack of equivalent validation is a significant difference. |
| **Insufficient sampling / optimization** | M3 (inadequate equilibration), M4 (insufficient sampling) | G-M2 (insufficient equilibration), G-M3 (insufficient sampling) | V-M1 (insufficient NELM), V-M2 (insufficient NSW) | Methodology. Universally silent in MD codes; partially implicit in VASP (detectable from iteration counts). |
| **Wrong model parameters (silent)** | T2 (wrong protonation), T4 (missing components) | G-T1 (wrong FF assignment), G-T3 (non-physical FF), G-T4 (subtle topology errors) | V-T1 (ENCUT too low), V-T2 (k-mesh inadequate), V-T3 (wrong POTCAR), V-T4 (wrong functional) | Theory. Universally silent. The most dangerous category across all frameworks because the simulation completes normally and produces results that appear valid but are physically incorrect. |
| **Methodology misconfiguration (silent)** | M2 (time step too large), M5 (wrong ensemble) | G-M1 (timestep too large), G-M4 (Berendsen thermostat) | V-M4 (wrong ISMEAR), V-M5 (SIGMA too large) | Methodology. Mostly silent, occasionally warned (GROMACS warns about Berendsen). |

#### DSL-specific failure modes (no cross-framework equivalent):

| Framework | Failure Mode | Why It Is Unique |
|:---|:---|:---|
| OpenMM | I4: Non-deterministic force summation artifacts | Specific to GPU parallel reduction in MD. GROMACS mitigates this differently. VASP's iterative SCF does not have this failure mode. |
| OpenMM | Custom reporter intercept gap (reporters cannot capture crash-step state) | OpenMM-specific architecture -- reporters fire between steps, not within them [OpenMM §5.3]. |
| GROMACS | G-A2: Domain decomposition failure | Specific to GROMACS's spatial decomposition parallelism. OpenMM uses different GPU parallelism. VASP uses band/k-point parallelism. |
| GROMACS | grompp preprocessing validation (catches errors before runtime) | GROMACS's unique compilation step. OpenMM has `createSystem()` (similar but less comprehensive). VASP has no equivalent preprocessing validator. |
| VASP | V-T1/V-T2: Silent basis set / k-point inadequacy | DFT-specific. MD codes have no basis set or k-point sampling. These are the most prevalent silent failures in DFT and have no MD analog. |
| VASP | V-A1: SCF non-convergence with exit code 0 | VASP-specific design choice to not set non-zero exit codes for non-convergence. MD codes crash on analogous failures (NaN propagation eventually causes exceptions). |
| VASP | Closed-source constraint (no custom instrumentation) | VASP-specific. OpenMM and GROMACS are open source and support custom instrumentation/reporters. |

---

## 4. Trace Completeness Assessment

### 4.1 OpenMM

**Observable state fraction:**
- With default reporters: approximately 30-40% of relevant execution state. StateDataReporter captures total energy scalars at configurable intervals; DCDReporter captures coordinates. Velocities, forces, per-force-group energies, and all implementation metadata are absent from default output. [OpenMM §1.8]
- With custom instrumentation: approximately 70-80%. Custom reporters can access positions, velocities, forces, per-force-group energy decomposition, all System/Integrator/Platform parameters, and external system metrics (GPU memory via pynvml). [OpenMM §5.2-5.3]

**Fundamentally hidden state:**
- GPU frame buffer contents during force evaluation. GPU global memory state between reporter callbacks is inaccessible from Python. [OpenMM §5.3]
- Sub-step integrator operations. The internal sequence of force evaluation, constraint solving, and velocity/position updates within a single `step()` call cannot be individually instrumented without modifying OpenMM's C++ core. [OpenMM §5.3]
- CUDA kernel execution order and thread scheduling. Non-deterministic GPU execution is invisible to the reporter API. [OpenMM §4.2]
- Random number generator internal state (accessible only via platform-specific checkpoint binary, not queryable at runtime). [OpenMM §1.5]
- Numerical precision loss accumulation. No mechanism to track rounding errors or denormalized float counts. [OpenMM §4.2]

**Custom instrumentation requirements for three-way fault classification:**
1. A custom ATHENA reporter implementing per-force-group energy decomposition at every reporting interval (via `context.getState(getEnergy=True, groups={i})`). [OpenMM §5.3, §5.4]
2. Adaptive reporting frequency that increases when anomalies are detected (energy approaching thresholds). [OpenMM §5.4]
3. Platform and precision metadata capture at simulation start. [OpenMM §4.1]
4. External GPU memory monitoring integration (pynvml or equivalent). [OpenMM §4.3]
5. Force magnitude distribution tracking for detecting "hot" atoms before NaN propagation. [OpenMM §5.3]

### 4.2 GROMACS

**Observable state fraction:**
- With default output: approximately 60-70%. The .edr provides complete energy decomposition by force type. The .log provides full parameter echo, performance metrics, and error messages. The .tpr captures complete input specification. Trajectories (.trr/.xtc) capture coordinates and optionally velocities/forces. [GROMACS §1.9]
- With custom instrumentation: approximately 75-85%. Setting `nstfout=1` and `nstenergy=1` would capture forces and energies at every step. GROMACS does not support user-defined reporters like OpenMM, so "custom instrumentation" means adjusting output frequencies and using analysis tools. [GROMACS §1.4, §4.2]

**Fundamentally hidden state:**
- Crash-step coordinates, velocities, and forces. The exact state at the moment of a fatal error is not preserved (GROMACS 2020+ writes crash-step coordinates to confout.gro but not velocities or forces). [GROMACS §6.5]
- GPU kernel execution details (kernel timing, thread scheduling, precision loss). [GROMACS §3.2]
- Internal constraint solver state (LINCS iteration history within a single step). Only the final result (converged or warning) is reported. [GROMACS §5.2]
- Domain decomposition communication contents (MPI message payloads are not logged). [GROMACS §3.2]
- Non-deterministic MPI reduction order effects on numerical results. [GROMACS §5.6]

**Custom instrumentation requirements for three-way fault classification:**
1. Energy monitoring via `panedr` with anomaly detection on .edr time series (drift, NaN, sudden jumps). [GROMACS §4.4]
2. LINCS warning parsing from .log with temporal correlation to energy trajectory. [GROMACS §5.2, §6.7]
3. grompp warning/note capture and classification. [GROMACS §7.1-7.2]
4. A GROMACS parameter classification table mapping all .mdp parameters to theory/implementation/boundary categories. [GROMACS §8.3]
5. Cross-file correlation engine merging .log events, .edr time series, and .tpr parameters. [GROMACS §8.3]

### 4.3 VASP

**Observable state fraction:**
- With default output: approximately 50-60%. vasprun.xml provides complete structured results (energies, forces, eigenvalues, convergence trajectory, parameters). OUTCAR adds timing, memory, warnings. OSZICAR adds compact convergence summaries. However, all output is per-ionic-step or per-SCF-step granularity -- there is no sub-SCF-step resolution. [VASP §4.1-4.3]
- With custom instrumentation: approximately 50-60% (essentially unchanged). VASP is closed-source with no custom reporter mechanism, no library mode, and no socket interface. The only instrumentation available is adjusting INCAR output control tags (LWAVE, LCHARG, LELF, LORBIT) and parsing the resulting output files. [VASP §7.1]

**Fundamentally hidden state:**
- Internal SCF solver state (Davidson iteration details, subspace rotation matrices, charge mixing internals beyond what OUTCAR reports). The REPORT file (VASP 6+) partially addresses this but is not comprehensively documented. [VASP §1.3, §7.1]
- Wavefunction coefficients during SCF iteration (only the converged wavefunctions are written to WAVECAR at the end of each ionic step, not intermediate SCF states). [VASP §1.2]
- MPI communication patterns and load balancing details (only summary information in stdout). [VASP §7.1]
- FFT grid aliasing and wrap-around error magnitudes. [VASP §7.4]
- PAW reconstruction error magnitudes. [VASP §7.4]
- Non-deterministic MPI reduction effects. [VASP §7.4]
- Any internal state that VASP does not choose to write -- ATHENA cannot add instrumentation to VASP's Fortran core. [VASP §7.1]

**Custom instrumentation requirements for three-way fault classification:**
1. vasprun.xml parser extracting SCF convergence trajectories, energy decomposition, forces, and parameters (existing: pymatgen `Vasprun`). [VASP §3.3, §7.3]
2. OUTCAR parser for warnings, timing, memory, and diagnostics (existing: pymatgen `Outcar`). [VASP §4.2, §7.3]
3. stdout/stderr parser for crash diagnostics and MPI errors (partially existing: pymatgen `custodian`). [VASP §5.3, §7.3]
4. An INCAR tag classification table mapping all parameters to theory/implementation/ambiguous. [VASP §2.4]
5. Domain-aware validation rules for silent failures: ENCUT vs. POTCAR ENMAX cross-check, k-mesh density heuristics, ISMEAR appropriateness for system type. [VASP §5.4-5.5, §6.3]
6. POTCAR identity and provenance tracking (partially available from vasprun.xml `<atominfo>`). [VASP §3.3]

### 4.4 Comparative Summary

| Metric | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|
| Observable state (default) | 30-40% | 60-70% | 50-60% |
| Observable state (max instrumentation) | 70-80% | 75-85% | 50-60% (ceiling due to closed source) |
| Custom reporter support | Yes (Python protocol, highly extensible) | No (output frequency adjustment only) | No (closed source, file I/O only) |
| Energy decomposition (default) | Total only | Per-force-type (native in .edr) | Per-SCF-step decomposition (native in vasprun.xml) |
| Crash-state preservation | Lost (Context invalid after crash) | Partial (confout.gro coords only, GROMACS 2020+) | Lost (truncated files) |
| Silent failure prevalence | Medium (methodology failures) | Medium (methodology + some theory) | High (theory + methodology, many with exit code 0) |

---

## 5. IR Generalizability Analysis

### 5.1 Common IR Core Elements

These elements generalize across all three frameworks and should form the universal IR schema.

**1. Timestamped Event Stream**
All three frameworks produce events that can be ordered temporally: energy values at step/iteration intervals, error/warning messages at specific points in execution, and state snapshots at configurable frequencies. The IR core must support a unified event type with: timestamp (step number, simulation time, or SCF/ionic iteration), event category (energy, state, error, metadata), and payload (the event data).

- OpenMM: Reporter callbacks at step intervals [OpenMM §1]
- GROMACS: .edr frames at nstenergy intervals, .log events at nstlog intervals [GROMACS §1.3, §3.2]
- VASP: SCF steps within ionic steps within the calculation [VASP §3.1]

**2. Energy Time Series**
All three frameworks produce energy as a function of simulation progress. The IR must represent energy trajectories as structured time series supporting: total energy, decomposed energy (per-force-group for MD, per-energy-component for DFT), and anomaly annotation (NaN, sudden jumps, drift).

- OpenMM: StateDataReporter total energy; custom reporter per-force-group [OpenMM §1.1, §5.3]
- GROMACS: .edr complete energy decomposition [GROMACS §4.1]
- VASP: vasprun.xml per-SCF and per-ionic energy decomposition [VASP §3.1, §3.2]

**3. Parameter Records**
All three frameworks have a declarative parameter specification that defines the calculation. The IR must represent the complete parameter set with per-parameter layer classification (theory / implementation / boundary).

- OpenMM: System + Integrator + Platform properties (queryable via API) [OpenMM §2, §4.1]
- GROMACS: .mdp parameters echoed in .log; .tpr via gmx dump [GROMACS §2.1, §3.2]
- VASP: INCAR parameters echoed in vasprun.xml `<incar>` and `<parameters>` [VASP §3.1, §3.2]

**4. Error Events**
All three frameworks produce error signals (exceptions, fatal error messages, warnings, crash exit codes). The IR must represent errors with: severity (fatal / warning / note), message text, source file/location, temporal context (when in execution), and preliminary fault classification (implementation / methodology / theory / ambiguous) with confidence level.

- OpenMM: Python exceptions (OpenMMException) [OpenMM §3.1]
- GROMACS: .log WARNING/NOTE/Fatal error messages, stderr [GROMACS §5.1]
- VASP: stdout/stderr crash messages, OUTCAR warnings [VASP §5.1-5.5]

**5. State Snapshots**
All three frameworks can produce snapshots of the physical system state at points during execution. The IR must represent state snapshots with: coordinates (always available), velocities (optional), forces (optional), and metadata (step number, simulation time, box vectors).

- OpenMM: Reporter output (DCD/PDB/PDBx/HDF5 for coordinates; custom for velocities/forces) [OpenMM §1.2-1.7]
- GROMACS: .trr (coordinates + optional velocities/forces), .xtc (coordinates only) [GROMACS §1.4, §1.5]
- VASP: XDATCAR (coordinates per ionic step), vasprun.xml (coordinates + forces per ionic step) [VASP §1.1, §1.3]

**6. Convergence Trajectory**
All three frameworks have iterative processes whose convergence behavior is diagnostic. The IR must represent convergence as a trajectory (sequence of residual/error values) with annotation for convergence status (converged / diverging / oscillating / stalled).

- OpenMM: Energy drift over simulation steps (derived from StateDataReporter) [OpenMM §7.2]
- GROMACS: Energy drift in .log statistics, constraint convergence in LINCS warnings [GROMACS §3.2, §5.2]
- VASP: SCF convergence trajectory (energy change per SCF step), ionic convergence (forces per ionic step) [VASP §3.1, §5.1, §5.2]

**7. Data Absence Records**
All three frameworks have gaps in their trace output (crash-state loss, blind spots between reporting intervals, unavailable metrics). The IR must represent what is NOT known as a first-class concept, since fault classification confidence depends on data completeness.

- OpenMM: Temporal gaps between reporter intervals; crash-state loss [OpenMM §6.5, §6.6]
- GROMACS: Crash-state gap (checkpoint may be thousands of steps before crash); methodology metrics not computed [GROMACS §6.5]
- VASP: Warnings not in vasprun.xml; internal SCF state hidden; crash truncates files [VASP §3.3, §7.1]

### 5.2 DSL-Specific Adapter Elements

These elements are unique to one framework and cannot be generalized into the common core, but the IR must still capture them through framework-specific adapters.

**OpenMM-specific:**
- Custom reporter extensibility metadata: The OpenMM adapter can capture richer data than default output because the reporter API allows arbitrary Python instrumentation. The adapter must track which reporters are active and what data they capture. [OpenMM §5.1-5.4]
- Platform comparison data: The ability to run identical simulations on Reference (double precision) vs. CUDA (single precision) platforms is a unique diagnostic capability. The adapter should support storing results from multiple platform runs for comparison. [OpenMM §7.1, I1]
- Force field compilation audit trail: The `createSystem()` boundary transforms theory-layer input into a System object. The adapter must capture both pre-compilation (ForceField XML, Topology) and post-compilation (System forces and parameters) state. [OpenMM §2.3]

**GROMACS-specific:**
- Checkpoint portability metadata: GROMACS .cpt files are architecture-independent and portable (unlike OpenMM's platform-specific checkpoints). The adapter must track checkpoint provenance (which version, which platform created it) and validate portability on load. [GROMACS §1.6]
- grompp preprocessing validation results: The grompp compilation step produces structured validation output (errors, warnings, notes) that has no equivalent in OpenMM or VASP. The adapter must capture grompp diagnostics as pre-execution audit events. [GROMACS §7.1-7.3]
- Runtime auto-tuning records: GROMACS auto-tunes nstlist, rlist, and PME parameters at runtime. The adapter must capture both the user-specified values and the runtime-adjusted values from the .log file, and flag discrepancies. [GROMACS §2.3]
- Per-force-type energy decomposition at native resolution: The .edr natively includes per-force-type decomposition (Bond, Angle, LJ, Coulomb, etc.) at every nstenergy interval, without any custom instrumentation. This is significantly richer than OpenMM's default output. [GROMACS §4.1]

**VASP-specific:**
- SCF convergence trajectory: The per-SCF-step energy change (dE), residual norm (d eps), and charge mixing metric (ncg) in OSZICAR/vasprun.xml are DFT-specific observables with no MD analog. The adapter must represent these as a nested convergence trajectory (SCF iterations within ionic iterations). [VASP §1.1, §3.1]
- Multi-file theory specification: VASP distributes theory across four input files (INCAR, POSCAR, POTCAR, KPOINTS). The adapter must fuse these into a unified specification record, unlike MD codes where the force field is a single file. [VASP §2.1, §8.2]
- Silent failure detection rules: The VASP adapter must implement domain-aware validation rules that VASP itself does not enforce: ENCUT vs. POTCAR ENMAX cross-check, k-mesh density heuristics, ISMEAR appropriateness, SIGMA magnitude checks. These rules have no equivalent in MD codes where analogous failures (wrong time step, insufficient equilibration) produce different symptoms. [VASP §5.4-5.5, §6.3]
- Closed-source ceiling: The VASP adapter cannot add instrumentation. It must work entirely with VASP's file output. The adapter interface must declare this constraint so the IR knows that certain data categories are fundamentally unavailable. [VASP §7.1]
- Electronic structure observables: Eigenvalues, DOS, band structure, charge density -- these DFT-specific outputs have no MD equivalent. The adapter must map these into the IR as theory-layer results for the Stage 3 (theoretical evaluation) audit. [VASP §1.2]

### 5.3 Adapter Interface Requirements

Each DSL adapter must provide the following to the common IR core. This defines the adapter contract.

**Required adapter outputs (mandatory for all adapters):**

| Interface Method | Description | OpenMM Implementation | GROMACS Implementation | VASP Implementation |
|:---|:---|:---|:---|:---|
| `extract_parameter_record()` | Returns complete parameter specification with per-parameter layer classification | Query System, Integrator, Platform via API; apply OpenMM classification table | Parse .log parameter echo or .tpr via gmx dump; apply GROMACS classification table | Parse vasprun.xml `<incar>` + `<parameters>`; fuse with POSCAR/POTCAR/KPOINTS metadata; apply VASP classification table |
| `extract_energy_series()` | Returns timestamped energy time series with available decomposition | Parse StateDataReporter CSV; call getState with force groups for decomposition | Read .edr via panedr into DataFrame | Parse vasprun.xml `<calculation>/<energy>` and `<scstep>/<energy>` nodes |
| `extract_state_snapshots()` | Returns available state snapshots (coordinates, optional velocities/forces) | Parse DCD/PDB/HDF5 trajectory files | Read .trr/.xtc via MDAnalysis | Parse XDATCAR or vasprun.xml `<structure>` nodes |
| `extract_error_events()` | Returns structured error events with severity and preliminary classification | Catch OpenMMException; parse exception message against known patterns | Parse .log for WARNING/NOTE/Fatal error patterns; parse stderr | Parse stdout/stderr for crash messages; parse OUTCAR for warnings |
| `extract_convergence_trajectory()` | Returns convergence metrics as time series | Derive from energy time series (drift computation) | Extract from .edr (energy drift) and .log (LINCS diagnostics) | Extract from OSZICAR/vasprun.xml (SCF dE series, ionic force series) |
| `extract_execution_metadata()` | Returns platform, version, timing, parallelization info | Query Platform API at runtime | Parse .log header and performance sections | Parse vasprun.xml `<generator>`, OUTCAR timing section, stdout |
| `declare_data_completeness()` | Returns a manifest of what data is/is not available, and what is fundamentally unrecoverable | Report which reporters are active; flag unavailable categories (forces, velocities, per-group energy) | Report output frequency settings; flag crash-state availability | Report available files; flag closed-source ceiling; flag silent failure detection capability |

**Optional adapter outputs (framework-specific, declared but not required for all adapters):**

| Interface Method | Applicable Frameworks | Description |
|:---|:---|:---|
| `extract_preprocessing_validation()` | GROMACS | grompp errors, warnings, notes |
| `extract_runtime_adjustments()` | GROMACS | Auto-tuned parameter values vs. user-specified values |
| `extract_scf_convergence()` | VASP | Per-SCF-step convergence metrics (dE, d eps, ncg) |
| `extract_electronic_structure()` | VASP | Eigenvalues, DOS, band structure |
| `validate_silent_failures()` | VASP (primary), GROMACS/OpenMM (secondary) | Domain-aware rule checks for unreported failures |
| `extract_force_field_compilation()` | OpenMM | Pre/post createSystem() audit data |
| `compare_platforms()` | OpenMM | Reference vs. CUDA/OpenCL comparison data |

---

## 6. Decision Gate 1 Assessment

### 6.1 Formal Evaluation

**Decision Gate 1 Question:** "Is the theory-implementation boundary cleanly API-enforced in ALL target frameworks?"

**Answer: No.** VASP fails this test.

- **OpenMM:** PASS. The ForceField/Topology/System chain (theory) and Platform/Context (implementation) are separated by API design. The `createSystem()` boundary is explicit and auditable. [OpenMM §2.4]
- **GROMACS:** CONDITIONAL PASS. The .mdp/topology (theory) and mdrun CLI (implementation) separation is structural, and the boundary parameters are identifiable. The .tpr merges both layers, but `gmx dump` + classification table recovers the separation. [GROMACS §2.3]
- **VASP:** FAIL. The INCAR flat namespace mixes theory and implementation without API-declared classification. Ambiguous parameters (PREC, LREAL, ALGO) create genuine cross-layer coupling. External classification is required. [VASP §2.4]

### 6.2 Recommendation: Accept VASP with External Classification Table

**Justification:**

1. **The boundary exists conceptually even though not API-declared.** Domain experts routinely and consistently distinguish between VASP physics parameters (GGA, ENCUT, ISPIN, LDAU) and execution parameters (NCORE, KPAR, LPLANE). This classification is stable, well-documented in the community, and does not require novel research to establish. [VASP §2.4]

2. **The classification table is finite and static.** The INCAR tag set comprises approximately 200-300 parameters total, of which 50-80 are commonly used [VASP §2.4]. Each parameter's classification (theory / implementation / ambiguous) can be determined once and maintained as a versioned lookup table. The table does not change between calculations -- it is a property of the VASP tag semantics, not of any specific calculation.

3. **External classification is a one-time engineering cost, not an ongoing research problem.** Building the INCAR classification table requires domain expertise but not novel research. The pymatgen and custodian projects have already partially performed this classification (custodian's error handlers implicitly classify parameters by the types of errors they produce). The remaining work is to formalize and complete the classification.

4. **Dropping VASP would narrow applicability to MD codes only, losing the DFT domain.** VASP is the most widely used DFT code in materials science. Excluding it would mean ATHENA can only demonstrate its approach on molecular dynamics (OpenMM, GROMACS), which share a fundamentally similar architecture (force field + integrator + trajectory). Including VASP tests whether ATHENA's IR can generalize to a structurally different computational domain (iterative eigenvalue problems, self-consistent field theory, plane-wave basis sets). This generality test is essential for validating ATHENA's architectural claims.

### 6.3 Risk Characterization

**What specific failure modes become harder to classify because of VASP's dirty boundary?**

| Failure Mode | Classification Difficulty Without Clean Boundary | Quantified Ambiguity |
|:---|:---|:---|
| V-A1: SCF non-convergence | **High.** With a clean boundary, the IR could determine whether SCF non-convergence is caused by implementation (ALGO choice = implementation if boundary were clean) or theory (functional inadequacy). With VASP's dirty boundary, ALGO is ambiguous -- for pathological systems, ALGO affects which physical solution the SCF converges to. | Estimated 10-20% of SCF non-convergence cases involve ALGO-dependent solution identity, based on community reports of systems where switching from ALGO=Normal to ALGO=All finds a different electronic ground state. [VASP §2.3, §5.1] |
| V-A2: PREC-induced inaccuracy | **High.** PREC simultaneously configures theory precision (FFT grid, augmentation grid) and implementation resources (memory allocation). A result that changes when switching from PREC=Normal to PREC=Accurate could be either an implementation failure (insufficient numerical precision for the algorithm) or a theory failure (the physical result is genuinely different at different basis set resolutions). | Estimated 5-10% of VASP calculations have PREC-dependent results that are scientifically meaningful (not just noise), particularly for systems with hard pseudopotentials or small unit cells. [VASP §2.3] |
| V-T1/V-T2: Silent basis/k-point inadequacy | **Medium.** Even with a clean boundary, these failures would be silent (VASP does not warn). The dirty boundary makes them slightly harder because ENCUT is listed as theory but is sometimes chosen based on computational budget (boundary behavior). | The classification table resolves this: ENCUT is tagged as "theory with implementation consequences," and the validation rule (ENCUT >= 1.3 * max(ENMAX_POTCAR)) provides an explicit check. [VASP §5.4] |
| V-M3: ALGO for pathological systems | **High.** ALGO is classified as implementation in the tag table, but for ~10-20% of systems (metals near magnetic transitions, strongly correlated materials), it affects which physical minimum the SCF finds. | The adapter must flag ALGO as ambiguous for systems where the domain heuristics detect potential pathology (ISPIN=2 with initial MAGMOM values suggesting competing magnetic states, LDAU=.TRUE., small band gap systems). [VASP §2.3, §6.1] |

**Aggregate risk:** For approximately 70-80% of standard VASP calculations (well-converged, non-magnetic, insulating or metallic with adequate smearing), the external classification table correctly classifies failures with the same confidence as OpenMM/GROMACS. For the remaining 20-30% (pathological SCF, strongly correlated, competing magnetic states), the classification confidence degrades due to ambiguous parameters. This degradation is bounded and can be explicitly flagged by the adapter.

### 6.4 Items Flagged for Adversarial Review

The following must be reviewed before the VASP adapter design is finalized:

1. **The INCAR classification table itself.** The complete tag-level classification (theory / implementation / ambiguous) for all ~200-300 INCAR parameters must be reviewed by a domain expert. Particular attention is required for the ambiguous parameters (PREC, LREAL, ALGO, ADDGRID, and context-dependent tags like ENCUT and NBANDS). The classification must be tested against real VASP failure cases to verify that it supports correct fault isolation.

2. **The "ambiguous for pathological systems" threshold.** The claim that ALGO is purely implementation for ~80% of systems and ambiguous for ~20% needs empirical validation. What system characteristics trigger the transition from "ALGO is implementation" to "ALGO is theory"? Can these characteristics be detected from the input specification alone?

3. **The closed-source ceiling's practical impact.** The claim that VASP's observable output is "sufficient for most fault isolation tasks" [VASP §7.4] needs stress-testing. Specifically: construct a set of VASP failure cases where the correct fault classification requires information that is not present in vasprun.xml + OUTCAR + stdout. How often does the closed-source ceiling prevent correct classification in practice?

4. **Whether the external classification table creates a maintenance burden that undermines ATHENA's generalizability claim.** If every new DSL requires a hand-curated classification table, ATHENA's "DSL-only environments" constraint becomes "DSL-only environments where someone has built a classification table." The adversarial review should assess whether classification tables can be partially automated (e.g., via LLM-assisted documentation analysis) or whether they are inherently manual.

5. **Cross-version stability of the classification.** VASP's defaults changed between versions 5 and 6 (e.g., LREAL behavior, ALGO defaults) [VASP §2.4]. The classification table must be version-aware. The adversarial review should assess the maintenance cost of version-specific tables and whether VASP version detection (from vasprun.xml `<generator>`) is reliable enough to select the correct table.

---

## 7. Implications for Downstream Steps

### 7.1 Step 3b (Requirements Refinement): Trace Capability Matrix Informs R1-R29 Coverage

The trace capability matrix (Section 1) directly informs which of the existing trace semantics requirements (R1-R29 from the requirements document) can be satisfied by each framework and which require additional instrumentation.

**Key findings for requirements coverage:**
- **Energy decomposition** (likely mapped to requirements around fault signal identification) is natively available in GROMACS (.edr per-force-type), natively available in VASP (vasprun.xml per-component), but requires custom instrumentation in OpenMM (per-force-group getState calls). Any requirement assuming energy decomposition is "always available" must be qualified.
- **Error event classification** requirements must account for the fact that all three frameworks use free-text error messages with no structured error codes. The IR cannot rely on typed error categories from the frameworks -- it must impose its own error taxonomy through pattern matching.
- **Crash-state preservation** is absent or severely limited across all three frameworks. Requirements assuming availability of the state at the moment of failure must be weakened to "state at the last reporting interval before failure."
- **Silent failure detection** requirements must distinguish between the MD case (silent methodology failures like insufficient equilibration) and the DFT case (silent theory failures like insufficient ENCUT). The detection mechanisms are fundamentally different: MD requires statistical analysis of output time series, while DFT requires rule-based cross-checks against input parameters.

### 7.2 Step 5 (Candidate IR Schemas): Constraints from Generalizability Analysis

The generalizability analysis (Section 5) places the following constraints on candidate IR schemas:

1. **The schema must support a common core + adapter extension architecture.** A monolithic schema covering all three frameworks would be unwieldy and would contain many unused fields per framework. The common core (Section 5.1) defines the universal elements; each adapter adds framework-specific extensions.

2. **The schema must represent temporal nesting.** VASP's SCF-within-ionic-step structure requires nested iteration representation that is not needed for MD codes. The common core's "timestamped event stream" must support hierarchical nesting (events within sub-events).

3. **The schema must support per-parameter layer classification as metadata.** Every parameter record must carry its theory/implementation/boundary classification. This classification comes from the adapter's classification table, not from the framework itself (since only OpenMM enforces it at the API level).

4. **The schema must represent data absence.** The `declare_data_completeness()` adapter method implies that the schema has first-class support for "this data category is unavailable" annotations. Data absence constrains the confidence of fault classification and must be propagated through the LFI's audit stages.

5. **The schema must not assume a specific temporal resolution.** OpenMM reports at step intervals (configurable), GROMACS reports at configurable step multiples, and VASP reports at SCF/ionic iteration boundaries. The schema's time axis must be generic enough to accommodate all three.

### 7.3 Adapter Architecture: Minimal Interface Requirements

The adapter contract defined in Section 5.3 establishes the minimal interface:

- **Seven mandatory methods** that every adapter must implement, providing the common data categories (parameters, energy, state, errors, convergence, metadata, completeness).
- **Seven optional methods** for framework-specific capabilities (preprocessing validation, runtime adjustments, SCF convergence, electronic structure, silent failure validation, force field compilation, platform comparison).
- **A classification table** that maps framework parameters to theory/implementation/boundary categories. This table is a data artifact, not code, and can be versioned independently of the adapter implementation.

The adapter architecture implies a two-layer design: a common IR ingestion layer that accepts the output of the mandatory methods, and framework-specific analysis modules that consume the optional method outputs. The LFI's three-stage audit operates on the common IR representation; framework-specific analysis supplements the audit with additional evidence where available.

---

**Sources:** All citations reference the three source documents by framework name and section number.
- [OpenMM §N] = openmm-trace-analysis.md, Section N
- [GROMACS §N] = gromacs-trace-analysis.md, Section N
- [VASP §N] = vasp-trace-analysis.md, Section N
