# OpenMM Trace Output System: Structured Analysis

**Date:** 2026-02-20
**Investigation:** Trace Semantics Engine IR Design (Priority 1)
**Purpose:** Characterize what OpenMM generates during simulation execution, mapping each output element to either the theory layer (what the scientist specifies) or the implementation layer (how the framework executes it).

**Evidence basis:** This analysis draws on the OpenMM 8.x Python API documentation (docs.openmm.org), the OpenMM source code (github.com/openmm/openmm, specifically `wrappers/python/openmm/app/`), the OpenMM User Guide (openmm.org/documentation), and documented failure patterns from the OpenMM GitHub issue tracker and community forums. Where claims rest on source code inspection, the specific source file is cited. Where claims rest on documented API behavior, the relevant documentation page is cited.

---

## 1. Reporter System Inventory

OpenMM's reporter system is the primary mechanism for trace output during simulation. Reporters are callback objects attached to a `Simulation` instance; they are invoked at regular intervals during `Simulation.step()` execution. The reporter contract is defined by the base interface requiring two methods: `describeNextReport()` and `report()`.

Source: `openmm/app/simulation.py` -- the `step()` method iterates through `self.reporters` and calls each reporter's methods at the appropriate intervals.

### 1.1 StateDataReporter

**Source:** `openmm/app/statedatareporter.py`
**Documentation:** docs.openmm.org, `openmm.app.StateDataReporter`

**Data emitted:**
- Simulation step number (integer)
- Elapsed time (simulation time, in picoseconds)
- Potential energy (kJ/mol)
- Kinetic energy (kJ/mol)
- Total energy (kJ/mol)
- Temperature (Kelvin)
- Volume (nm^3, for periodic systems)
- Density (g/mL, for periodic systems)
- Simulation speed (ns/day or steps/second)
- Elapsed wall-clock time
- Remaining estimated time

**Format:** CSV-like text output to a file object or `sys.stdout`. The first line is a header row with column names prefixed by `#`. Subsequent lines are comma-separated values. The output stream is configurable (any Python file-like object).

**Configurability:** Each data column is individually togglable via boolean constructor parameters: `step`, `time`, `potentialEnergy`, `kineticEnergy`, `totalEnergy`, `temperature`, `volume`, `density`, `speed`, `elapsedTime`, `remainingTime`. The `reportInterval` parameter controls how frequently (in steps) the reporter fires. An `append` parameter controls whether the output file is opened in append mode. A `separator` parameter allows changing the delimiter (default: comma).

**Layer classification: Mixed.**
- Temperature, energy values, volume, density are **theory-layer observables** -- they report on the physical state of the system as determined by the force field and equations of motion.
- Speed, elapsed time, remaining time are **implementation-layer metadata** -- they report on computational execution performance.
- Step number and simulation time are **boundary quantities** -- they bridge the theory-implementation divide (the integrator's time step is a theory choice; the step count is an implementation state variable).

**Critical for ATHENA:** StateDataReporter is the primary source for detecting energy anomalies (NaN, explosion). Its `totalEnergy` field is the first signal of numerical instability. However, it only reports at its configured interval -- events between reporting intervals are invisible.

### 1.2 DCDReporter

**Source:** `openmm/app/dcdreporter.py`
**Documentation:** docs.openmm.org, `openmm.app.DCDReporter`

**Data emitted:** Atomic coordinates (positions) for all particles at each reporting interval. Optionally includes periodic box vectors. Written in the DCD binary trajectory format (CHARMM/NAMD compatible).

**Format:** Binary DCD format. Not human-readable. Requires trajectory analysis tools (MDTraj, MDAnalysis, VMD) for parsing.

**Configurability:** `reportInterval` (steps between frames), `append` (append to existing file), `enforcePeriodicBox` (whether to wrap coordinates into the periodic box -- default True).

**Layer classification: Theory layer.** Atomic positions are physical observables. The `enforcePeriodicBox` option straddles the boundary -- periodic wrapping is a representational choice, but periodic box vectors themselves are theory-layer (they define the simulation cell).

**Limitation for ATHENA:** Binary format requires additional tooling to parse. The DCD format does not include energy data, force data, or metadata about the simulation state beyond coordinates and box vectors. Frame timestamps are step-count based, not wall-clock based.

### 1.3 PDBReporter

**Source:** `openmm/app/pdbreporter.py`
**Documentation:** docs.openmm.org, `openmm.app.PDBReporter`

**Data emitted:** Atomic coordinates in PDB text format. Includes atom names, residue names, chain identifiers, and coordinates. Each frame is written as a MODEL/ENDMDL block.

**Format:** PDB text format (Protein Data Bank standard). Human-readable but verbose. Each atom line follows the PDB column specification (fixed-width fields).

**Configurability:** `reportInterval`, `enforcePeriodicBox`. Less configurable than DCDReporter. PDB format has inherent precision limitations (coordinates stored to 3 decimal places in Angstroms, i.e., 0.001 Angstrom resolution).

**Layer classification: Theory layer.** Same as DCDReporter -- reports physical observables (coordinates). The PDB format itself imposes a precision loss relative to the simulation's internal representation, which is an implementation artifact of the file format, not the simulation.

### 1.4 PDBxReporter

**Source:** `openmm/app/pdbxreporter.py`
**Documentation:** docs.openmm.org, `openmm.app.PDBxReporter`

**Data emitted:** Atomic coordinates in PDBx/mmCIF format. Functionally similar to PDBReporter but uses the newer mmCIF format, which has no column-width limitations and better precision.

**Format:** PDBx/mmCIF text format. Human-readable, structured as key-value data blocks.

**Configurability:** `reportInterval`, `enforcePeriodicBox`.

**Layer classification: Theory layer.** Same as PDBReporter.

### 1.5 CheckpointReporter

**Source:** `openmm/app/checkpointreporter.py`
**Documentation:** docs.openmm.org, `openmm.app.CheckpointReporter`

**Data emitted:** Complete simulation state checkpoint. This includes positions, velocities, periodic box vectors, integrator state (including thermostat/barostat internal variables), and random number generator state. Written as an opaque binary blob via `Context.createCheckpoint()`.

**Format:** Binary. Platform-specific and version-specific. Not portable across different OpenMM versions or platforms. The format is intentionally opaque -- it is a serialized memory dump of the Context's internal state.

**Configurability:** `reportInterval`, `writeCheckpoint` (vs. `writeState` -- checkpoint is platform-specific binary, state is portable XML).

**Layer classification: Implementation layer.** Checkpoint data is explicitly implementation-dependent. The OpenMM documentation warns that checkpoints are not portable across platforms (e.g., a CUDA checkpoint cannot be loaded on the CPU platform). The internal integrator state (thermostat variables, RNG seeds) is implementation detail that the user does not directly specify. However, the checkpoint *contains* theory-layer data (positions, velocities) in an implementation-specific encoding.

**Critical for ATHENA:** CheckpointReporter is the primary mechanism for state recovery after failures. The platform-dependence of checkpoints is a significant complication for deterministic auditing -- the same simulation state may be represented differently across platforms.

### 1.6 XTCReporter (via mdtraj)

**Source:** Not part of core OpenMM. Provided by the `mdtraj` library (`mdtraj.reporters.XTCReporter`).
**Documentation:** mdtraj.org

**Data emitted:** Atomic coordinates in XTC compressed trajectory format (GROMACS-compatible). Lossy compression of coordinates with configurable precision.

**Format:** XTC binary format. Compressed, so significantly smaller than DCD for equivalent trajectories.

**Configurability:** `reportInterval`, precision (compression level).

**Layer classification: Theory layer** (observables), but the lossy compression introduces an implementation-layer transformation. The precision parameter explicitly controls how much information is discarded.

### 1.7 HDF5Reporter (via mdtraj)

**Source:** `mdtraj.reporters.HDF5Reporter`
**Documentation:** mdtraj.org

**Data emitted:** Positions, velocities (optional), forces (optional), potential energy, kinetic energy, temperature, box vectors, simulation time. All stored in a single HDF5 container.

**Format:** HDF5 binary format. Self-describing, supports metadata, random access.

**Configurability:** `reportInterval`, `coordinates` (bool), `time` (bool), `cell` (bool), `potentialEnergy` (bool), `kineticEnergy` (bool), `temperature` (bool), `velocities` (bool), `atomSubset` (report only specific atoms).

**Layer classification: Mixed.** This is the richest single-reporter format. It combines theory-layer observables (energy, temperature, coordinates, forces) with simulation metadata (time). The `velocities` option is notable -- velocities are theory-layer state (they determine kinetic energy) but are rarely reported by default.

**Critical for ATHENA:** HDF5Reporter is the closest existing reporter to what a comprehensive trace capture system would need. Its ability to optionally include forces is particularly valuable -- forces are the direct output of the force field evaluation and provide theory-layer diagnostic information not available from any core OpenMM reporter.

### 1.8 Summary Table

| Reporter | Source | Format | Data Category | Layer | Key for ATHENA |
|:---|:---|:---|:---|:---|:---|
| StateDataReporter | Core OpenMM | CSV text | Thermodynamic scalars, performance | Mixed | Energy anomaly detection, NaN signal |
| DCDReporter | Core OpenMM | DCD binary | Coordinates, box vectors | Theory | Trajectory analysis |
| PDBReporter | Core OpenMM | PDB text | Coordinates, topology | Theory | Human-readable snapshots |
| PDBxReporter | Core OpenMM | mmCIF text | Coordinates, topology | Theory | Higher precision than PDB |
| CheckpointReporter | Core OpenMM | Binary blob | Full simulation state | Implementation | State recovery, but platform-dependent |
| XTCReporter | mdtraj | XTC binary | Coordinates (lossy) | Theory | Compact trajectory storage |
| HDF5Reporter | mdtraj | HDF5 binary | Coordinates, energy, forces, velocities | Mixed | Richest single-reporter format |

---

## 2. Theory-Implementation API Boundary

### 2.1 Theory Layer

The theory layer in OpenMM consists of the scientific specification: what physical system is being simulated and under what physical laws.

**ForceField** (`openmm.app.ForceField`)
- Encodes the potential energy function: which force terms exist (bonds, angles, dihedrals, nonbonded), their functional forms, and their parameters.
- Loaded from XML parameter files (e.g., `amber14-all.xml`, `charmm36.xml`). These files define atom types, residue templates, and force parameters.
- The scientist's theory choice: "I believe this force field accurately represents the physics of my system."
- Source: `openmm/app/forcefield.py`, OpenMM User Guide Ch. 3 ("Force Fields").

**Topology** (`openmm.app.Topology`)
- Describes the molecular structure: atoms, bonds, residues, chains, periodic box dimensions.
- Typically loaded from PDB/PDBx files or constructed programmatically.
- The scientist's system specification: "This is the molecule I want to simulate."
- Source: `openmm/app/topology.py`.

**System** (`openmm.openmm.System`)
- Contains the particle masses, force objects (with parameters), and constraints.
- This is the compiled representation of "force field + topology = computational physics specification."
- The scientist can add custom forces (via `CustomBondForce`, `CustomNonbondedForce`, etc.) that encode arbitrary energy expressions as string formulas.
- Source: OpenMM C++ core, Python wrapper in `openmm/openmm.py`.

**Integrator choices** (`openmm.LangevinMiddleIntegrator`, `openmm.VerletIntegrator`, etc.)
- The equations of motion: how the system evolves in time.
- Parameters like temperature, friction coefficient (for Langevin dynamics), and time step size are theory-layer choices -- they define the thermodynamic ensemble and temporal resolution.
- Source: OpenMM C++ core, Python wrappers.

**Barostat** (`openmm.MonteCarloBarostat`)
- Pressure control: defines the target pressure and coupling frequency.
- Theory-layer: the scientist specifies what thermodynamic ensemble to sample (NPT vs NVT).

### 2.2 Implementation Layer

The implementation layer consists of how the computation is executed, which ideally should not affect the scientific results (but in practice can).

**Platform** (`openmm.Platform`)
- The compute backend: Reference, CPU, CUDA, OpenCL.
- Selected explicitly by the user or auto-detected.
- The Reference platform is the correctness baseline (pure C++, no optimizations, full double precision).
- CUDA and OpenCL platforms use GPU acceleration with different numerical implementations.
- Source: `openmm/openmm.py`, OpenMM User Guide Ch. 8 ("Platforms").

**Platform properties:**
- `Precision`: "single", "mixed", or "double" (CUDA/OpenCL only). "Mixed" uses single precision for forces, double for accumulation. Default: "single" on CUDA.
- `DeviceIndex`: which GPU to use.
- `CudaCompiler`: path to the CUDA compiler.
- `DeterministicForces`: whether to use deterministic force summation (slower but reproducible). Default: False on CUDA.
- These are implementation choices that should not affect the physics but can affect numerical stability.

**Context** (`openmm.Context`)
- The execution container that binds a System + Integrator + Platform together.
- Holds the current state (positions, velocities, forces) on the compute device.
- The Context is the bridge between theory and implementation: it takes theory-layer specifications and instantiates them on a specific compute backend.
- Source: OpenMM C++ core, Python wrapper.

**Parallelization:**
- Thread count (CPU platform).
- GPU device selection (CUDA/OpenCL).
- Multi-GPU parallelization (via `CudaDeviceIndex` property, comma-separated device list).
- These are purely implementation choices.

### 2.3 The Boundary: ForceField.createSystem()

`ForceField.createSystem(topology, **kwargs)` is the critical compilation step where theory-layer specifications are translated into a computational System object.

**Source:** `openmm/app/forcefield.py`, method `createSystem()`.

**What it does:**
1. Matches atoms in the Topology to atom types in the ForceField's residue templates.
2. Assigns force field parameters (charges, Lennard-Jones parameters, bond/angle/dihedral parameters) to each atom/interaction.
3. Creates Force objects (HarmonicBondForce, HarmonicAngleForce, PeriodicTorsionForce, NonbondedForce, etc.) with the assigned parameters.
4. Applies constraint algorithms (SHAKE/SETTLE for hydrogen bonds if `constraints=HBonds`).
5. Configures nonbonded method (NoCutoff, CutoffPeriodic, PME, etc.) with cutoff distance and PME parameters.
6. Returns a `System` object containing all forces and particles.

**Key parameters crossing the boundary:**
- `nonbondedMethod`: CutoffNonPeriodic, CutoffPeriodic, PME, Ewald, NoCutoff. This is a methodology choice (how to handle long-range electrostatics) that has both theory and implementation implications.
- `nonbondedCutoff`: Distance cutoff for nonbonded interactions. Theory-layer (affects the physics).
- `constraints`: None, HBonds, AllBonds, HAngles. Constraining bonds is a methodology choice that affects dynamics but enables larger time steps.
- `rigidWater`: Whether water molecules are rigid. Theory/methodology boundary.
- `ewaldErrorTolerance`: Controls PME grid spacing. Straddles theory-implementation: it affects accuracy of the electrostatic computation (theory-relevant) but is an algorithmic parameter (implementation detail).
- `hydrogenMass`: Hydrogen mass repartitioning for larger time steps. Affects dynamics (theory) but is a computational trick (implementation).

**What is lost/transformed:**
1. **Atom type assignment is irreversible.** Once parameters are assigned, the System does not retain the original atom type labels from the force field XML. You cannot query "which atom type was atom 1234 assigned?" from the System alone. The mapping from force field types to specific parameters is baked in.
2. **Residue template matching is not recorded.** If an atom matches multiple templates, the first match wins, but no audit trail is produced. A mismatched template (wrong protonation state, missing atom) produces a hard error at this stage, which is good for ATHENA -- the failure is surfaced as an exception.
3. **Custom modifications are merged.** If the user adds `CustomNonbondedForce` objects or modifies System forces after `createSystem()`, these modifications are not distinguished from the original force field parameters in the System object.
4. **Constraint application is recorded.** Constraints are enumerable from the System (`system.getNumConstraints()`, `system.getConstraintParameters(i)`), so this transformation is auditable.
5. **PME parameters are auto-computed.** The `ewaldErrorTolerance` is translated into specific grid dimensions and Ewald coefficient, but these derived values are accessible via `NonbondedForce.getPMEParameters()`.

**Assessment for deterministic auditing:**
The `ForceField.createSystem()` boundary is *partially* clean. The positive: it produces a well-defined System object whose contents (forces, parameters, constraints, masses) are fully queryable via the OpenMM API. The negative: the atom type assignment process is one-directional and does not preserve the mapping trail. For ATHENA's purposes, the System object after `createSystem()` is the auditable theory specification. The question of whether the force field parameters were correctly applied to the topology requires comparing the System's parameters against the original force field XML and topology -- a tractable but non-trivial reverse-engineering step.

### 2.4 Boundary Cleanliness Assessment

**Strengths:**
- OpenMM enforces a clear API separation. You cannot accidentally mix force field parameter specification with GPU kernel code. The `System` object is a pure physics description; the `Platform` is a pure execution backend.
- The `Context.getState()` API provides a unified way to query physical state (positions, velocities, forces, energies) regardless of which Platform is executing. This is a strong abstraction boundary.
- Forces are individually queryable. You can enumerate all Force objects in a System, query their parameters, and identify which force terms are active. This supports the LFI's Stage 1 (implementation audit).

**Weaknesses:**
- The atom type assignment trail is lost at the `createSystem()` boundary. This matters for theory-layer auditing: if the wrong atom types were assigned (due to ambiguous topology), the error is silent after `createSystem()` unless independently checked.
- Platform-level numerical differences are real but not surfaced. Running the same System on CUDA (single precision) vs. Reference (double precision) can produce different trajectories, and the divergence is not reported anywhere in the standard output.
- The `DeterministicForces` property defaults to False on CUDA, meaning force summation order (and therefore results) can vary between runs on the same hardware. This non-determinism is an implementation-layer property that affects theory-layer observables.

---

## 3. Exception and Error Exposure

### 3.1 Exception Types

OpenMM surfaces errors through Python exceptions. The primary exception class is `OpenMMException` (C++-level, mapped to Python via SWIG). Common exception scenarios:

**Theory-layer errors (surfaced at System construction time):**

| Exception Context | Typical Message | Root Cause Category |
|:---|:---|:---|
| `ForceField.createSystem()` template matching failure | `"No template found for residue X"` | Theory: wrong force field for this molecule, or topology has unparameterized residues |
| `ForceField.createSystem()` missing parameters | `"No parameters defined for atom type X"` | Theory: incomplete force field coverage |
| Invalid constraint specification | `"Cannot add constraint between particles in different molecules"` | Theory/methodology: impossible physical constraint |
| Custom force expression parse error | `"Error parsing expression: ..."` | Theory: malformed energy function |

**Implementation-layer errors (surfaced at execution time):**

| Exception Context | Typical Message | Root Cause Category |
|:---|:---|:---|
| `Context` creation with incompatible Platform | `"Error initializing CUDA"` or `"No compatible GPU found"` | Implementation: hardware/driver issue |
| Out of GPU memory | `"Error creating context: out of memory"` | Implementation: system too large for available hardware |
| NaN detected during integration | `"Particle coordinate is NaN"` or energy becomes NaN | Ambiguous: can be theory (bad parameters), methodology (too large time step), or implementation (precision) |
| Constraint failure | `"Constraint tolerance not met"` | Ambiguous: can be methodology (initial structure too far from equilibrium) or theory (incompatible constraints) |

**Methodology-layer errors (NOT surfaced by OpenMM -- invisible):**

OpenMM does not detect or report:
- Insufficient equilibration (system not yet at equilibrium when production data is collected)
- Inadequate sampling (simulation too short to converge observables)
- Wrong ensemble (NVT when NPT was needed)
- Correlation between samples (reporting interval too short relative to decorrelation time)
- Selection of inappropriate collective variables for analysis

This is a critical finding for ATHENA: **methodology-layer failures are almost entirely invisible to OpenMM's error reporting system.** The framework has no concept of "scientific adequacy" -- it faithfully executes whatever simulation is specified, regardless of whether the specification is scientifically meaningful.

### 3.2 The NaN Detection Mechanism

OpenMM's NaN handling is limited. The framework does not proactively check for NaN values during integration. NaN propagation is detected in specific scenarios:

1. **StateDataReporter:** If energy is NaN, it will be reported as `nan` in the output. But the reporter only fires at its configured interval -- NaN can propagate silently between intervals.
2. **Reporter-level check:** Some reporters (e.g., StateDataReporter) do not crash on NaN -- they report it as text.
3. **Context.getState():** Calling `getState(getEnergy=True)` will return NaN values without raising an exception. The user must explicitly check.
4. **Platform-level detection:** The CUDA platform may produce NaN from denormalized floats or overflow in single precision. The Reference platform (double precision) is much less susceptible but can still produce NaN from genuinely bad parameters (e.g., overlapping atoms with strong repulsive forces).

Source: OpenMM GitHub issues (multiple threads on NaN energy, including discussions in openmm/openmm issues #2143, #3093, and others).

### 3.3 Error Reporting Gaps

**What OpenMM does NOT report:**
- Which specific force term produced a NaN or extreme value (the energy is reported as a total, not per-force-term, unless the user explicitly calls `getState(getEnergy=True, groups={i})` for each force group).
- Whether numerical precision was the cause of divergence (no precision diagnostic exists).
- Memory usage statistics (no built-in memory reporting; GPU memory must be queried externally via CUDA APIs).
- Thread/GPU utilization metrics.
- Force evaluation timing breakdown (though this is available via `Context.getState(getEnergyParameterDerivatives=True)` for some use cases).

**What is significant for ATHENA:** OpenMM's error surface is sparse. The framework throws exceptions for clearly invalid states (missing parameters, platform failures) but is silent about the most common and most difficult-to-diagnose failures: gradual energy drift, inadequate sampling, and NaN from numerical precision issues. An ATHENA Trace Semantics Engine targeting OpenMM would need to actively instrument the simulation (via custom reporters or external monitoring) rather than relying on the framework's default error reporting.

---

## 4. Execution Metadata

### 4.1 Platform and Device Information

The following execution metadata is accessible through the OpenMM API:

**Queryable at any time via `Platform` and `Context` APIs:**

| Metadata | API Call | Category |
|:---|:---|:---|
| Platform name | `context.getPlatform().getName()` | Implementation |
| Platform speed (relative) | `Platform.getPlatformByName(name).getSpeed()` | Implementation |
| Platform properties | `platform.getPropertyValue(context, name)` | Implementation |
| Precision mode | `platform.getPropertyValue(context, 'Precision')` | Implementation |
| CUDA device index | `platform.getPropertyValue(context, 'CudaDeviceIndex')` | Implementation |
| CUDA compiler | `platform.getPropertyValue(context, 'CudaCompiler')` | Implementation |
| OpenCL device index | `platform.getPropertyValue(context, 'OpenCLDeviceIndex')` | Implementation |
| Deterministic forces enabled | `platform.getPropertyValue(context, 'DeterministicForces')` | Implementation |
| Number of platforms available | `Platform.getNumPlatforms()` | Implementation |
| OpenMM version | `openmm.version.full_version` | Implementation |

**Queryable from the System:**

| Metadata | API Call | Category |
|:---|:---|:---|
| Number of particles | `system.getNumParticles()` | Theory |
| Number of constraints | `system.getNumConstraints()` | Theory/Methodology |
| Number of forces | `system.getNumForces()` | Theory |
| Force group assignments | `force.getForceGroup()` | Theory (user-assigned) |
| Periodic box vectors | `system.getDefaultPeriodicBoxVectors()` | Theory |
| Particle masses | `system.getParticleMass(i)` | Theory |
| CMMotionRemover frequency | `cmmotion.getFrequency()` | Methodology |

**Queryable from the Integrator:**

| Metadata | API Call | Category |
|:---|:---|:---|
| Time step size | `integrator.getStepSize()` | Theory/Methodology |
| Constraint tolerance | `integrator.getConstraintTolerance()` | Methodology |
| Temperature (Langevin) | `integrator.getTemperature()` | Theory |
| Friction coefficient (Langevin) | `integrator.getFriction()` | Theory |

### 4.2 What is NOT Accessible

- **GPU memory usage:** Not queryable through OpenMM APIs. Requires external CUDA/OpenCL memory queries.
- **Per-kernel timing:** OpenMM does not expose timing breakdowns for individual force evaluations, constraint solving, or integration steps. Profiling requires external CUDA profiling tools (nvprof, nsys).
- **Thread utilization:** The CPU platform's thread count is configurable but utilization is not reported.
- **Numerical precision diagnostics:** No mechanism to report precision loss, denormalized float counts, or rounding error accumulation.
- **Force evaluation counts:** The total number of force evaluations is not tracked (it equals the number of steps for simple integrators, but multiple time stepping or Monte Carlo moves complicate this).

### 4.3 Assessment for Trace Capture

OpenMM provides good coverage of what platform is being used and how it is configured, but poor coverage of how the platform is performing during execution. An ATHENA trace capture system would need to supplement OpenMM's metadata with:
1. External GPU memory monitoring (via `pynvml` or similar).
2. Per-force-group energy decomposition at each reporting interval (via `getState(groups=...)` calls).
3. Custom timing instrumentation around `step()` calls.
4. Precision diagnostics (comparing Reference platform results against CUDA/OpenCL results for representative configurations).

---

## 5. Custom Reporter Extensibility

### 5.1 The Reporter API Contract

OpenMM's reporter system is extensible via a simple Python protocol. Any Python object with two methods can serve as a reporter:

**`describeNextReport(simulation)`**
- Called by `Simulation.step()` before each step to determine when this reporter next needs to fire.
- Returns a tuple: `(steps_until_next_report, positions_needed, velocities_needed, forces_needed, energy_needed)` -- where each boolean indicates what state data the reporter will require.
- The `steps_until_next_report` value allows reporters to fire at non-uniform intervals.
- Some versions extend this to include `enforcePeriodicBox` as a sixth element.

Source: `openmm/app/simulation.py`, the `_generate_reports()` or `step()` method.

**`report(simulation, state)`**
- Called when the reporter fires.
- Receives the `Simulation` object and a `State` object (obtained via `Context.getState()` with the flags specified in `describeNextReport`).
- The reporter can do anything with this data: write to files, log to databases, trigger callbacks, compute derived quantities, etc.

### 5.2 What Custom Reporters Can Access

Through the `Simulation` object:
- `simulation.context` -- the full Context, allowing arbitrary API calls.
- `simulation.topology` -- the molecular Topology.
- `simulation.system` -- the System (all forces and parameters).
- `simulation.integrator` -- the Integrator.
- `simulation.currentStep` -- the current step number.

Through the `State` object:
- `state.getPositions()` -- all particle positions.
- `state.getVelocities()` -- all particle velocities.
- `state.getForces()` -- forces on all particles.
- `state.getPotentialEnergy()` -- total potential energy.
- `state.getKineticEnergy()` -- total kinetic energy.
- `state.getPeriodicBoxVectors()` -- current box dimensions.
- `state.getPeriodicBoxVolume()` -- current box volume.
- `state.getParameters()` -- context parameters (e.g., lambda values for alchemical simulations).
- `state.getEnergyParameterDerivatives()` -- derivatives of energy with respect to context parameters.

Through per-force-group energy decomposition:
- `context.getState(getEnergy=True, groups={i})` -- energy contribution from force group `i` only.
- This allows decomposing the total energy into contributions from bonds, angles, dihedrals, nonbonded, etc.

### 5.3 Hooks for Custom Instrumentation

**What is possible:**
1. **Per-force-group energy tracking:** A custom reporter can call `getState(groups={i})` for each force group to produce an energy decomposition at every reporting interval. This is the most valuable instrumentation for ATHENA -- it allows identifying which force term is producing anomalous energies.
2. **Force vector monitoring:** Requesting forces in `describeNextReport` allows tracking per-atom force magnitudes. Sudden force spikes indicate specific problematic interactions.
3. **Velocity monitoring:** Tracking per-atom velocities to detect "hot" atoms before they cause NaN energies.
4. **Parameter derivative tracking:** For alchemical simulations, tracking `dE/dlambda` values.
5. **Adaptive reporting:** A custom reporter can change its reporting interval dynamically (e.g., increase frequency when energy values approach dangerous thresholds).
6. **External system monitoring:** A reporter can query GPU memory, CPU load, or other system metrics during `report()`.

**What is NOT possible via the reporter API:**
1. **Sub-step instrumentation:** Reporters fire between steps, not within them. The internal integrator operations (force evaluation, constraint solving, velocity update) cannot be individually instrumented without modifying OpenMM's C++ core.
2. **Kernel-level profiling:** GPU kernel execution cannot be profiled from within a reporter.
3. **Intercepting exceptions:** If a step throws an exception, the reporter is not called for that step. The exception propagates to the `step()` caller. This means a reporter cannot capture the state immediately before a crash -- it can only capture state at the last successful reporting interval.
4. **Modifying the simulation:** Reporters are read-only by convention (though not enforced). A reporter that modifies the Context's state during `report()` will have unpredictable effects.

### 5.4 Assessment for ATHENA

The reporter API is well-designed for extensibility. A custom ATHENA reporter could capture significantly more trace data than any default reporter, including per-force-group energy decomposition, force magnitude distributions, and velocity statistics. The main gap is sub-step instrumentation: when a simulation crashes mid-step, the last known state is from the previous reporting interval, which may be hundreds or thousands of steps before the failure. For NaN propagation, this means the root cause may have occurred long before it was detectable.

The most promising instrumentation strategy would be an adaptive reporter that:
1. Reports per-force-group energies at every interval.
2. Monitors total energy for drift or anomalies.
3. Increases reporting frequency when anomalies are detected.
4. Records platform and precision metadata at the start of each simulation.

---

## 6. Failure Walkthrough: NaN Energy

### 6.1 Scenario Setup

A researcher sets up a protein simulation using:
- Force field: Amber14 (`amber14-all.xml`) with TIP3P water (`amber14/tip3p.xml`)
- System: 50,000 atoms (protein + solvent)
- Integrator: LangevinMiddleIntegrator, 300 K, 1.0/ps friction, 2 fs time step
- Constraints: HBonds (hydrogen bonds constrained via SHAKE)
- Platform: CUDA, single precision
- Nonbonded: PME, 1.0 nm cutoff
- Reporters: StateDataReporter (every 1000 steps), DCDReporter (every 10000 steps), CheckpointReporter (every 100000 steps)

After 50,000 steps (100 ps), energy begins diverging. At step 52,347, a particle coordinate becomes NaN.

### 6.2 What Each Reporter Emits

**StateDataReporter output (last few entries):**
```
#"Step","Time (ps)","Potential Energy (kJ/mol)","Temperature (K)","Speed (ns/day)"
49000,98.0,-234567.89,301.2,150.3
50000,100.0,-234501.23,300.8,150.1
51000,102.0,-233890.12,305.4,149.8
52000,104.0,-198234.56,412.7,149.5
```

The reporter at step 51000 shows a temperature spike (305.4 K, slightly elevated). At step 52000, the potential energy has jumped dramatically (-198234 vs -234501) and temperature is 412.7 K. The NaN occurred at step 52347, but the reporter will not fire until step 53000 -- if the simulation has already crashed, the step 53000 report never happens.

**Information available from StateDataReporter:**
- Energy was normal at step 50000.
- Energy was already anomalous at step 52000 (2000 steps after last normal report).
- The exact step where instability began is unknown (somewhere between step 50001 and step 52000).
- There is no indication of *which* force term caused the energy spike.
- The temperature spike correlates with the energy anomaly but does not indicate cause.

**DCDReporter output:**
- Last frame written at step 50000 (the reporter fires every 10000 steps).
- The coordinates at step 50000 are normal.
- No frame exists between step 50000 and the crash. The 2347 steps of divergence are completely unrecorded in the trajectory.

**CheckpointReporter output:**
- Last checkpoint at step 0 (the reporter fires every 100000 steps).
- The checkpoint is too old to be useful for diagnosing the crash.

### 6.3 Exceptions Thrown

When the NaN particle coordinate is encountered:

**Scenario A: NaN propagates to Context.getState()**
If the simulation continues running (NaN does not immediately cause a crash on CUDA), the next call to `context.getState()` (triggered by a reporter) returns NaN for energy values. No exception is raised. The StateDataReporter writes "nan" to the output. The simulation continues producing garbage.

**Scenario B: NaN causes a CUDA kernel error**
If the NaN triggers an invalid memory access or arithmetic exception in a CUDA kernel, the Context becomes invalid. The next `step()` call raises an `OpenMMException` with a CUDA error message. The Python call stack shows:
```
OpenMMException: Error launching CUDA kernel: ...
```
or
```
OpenMMException: Particle coordinate is NaN. For more information, see https://...
```

The exception message does not indicate:
- Which particle has the NaN coordinate.
- Which force evaluation produced the NaN.
- Whether precision was a contributing factor.
- What the state was immediately before the NaN.

### 6.4 State Recovery

**What is recoverable:**
1. The last StateDataReporter output (step 52000) showing anomalous energy.
2. The last DCD frame (step 50000) showing normal coordinates.
3. The last checkpoint (step 0, too old to be useful in this scenario).
4. The System object (all force parameters, queryable).
5. The Platform and precision settings (queryable from Context, if Context is still valid).
6. The Topology (atom/residue/bond information).

**What is lost:**
1. The state at the moment of failure (step 52347). If the Context is invalid, positions/velocities/forces at the failure point are unrecoverable.
2. The state between the last normal report (step 50000) and the failure (step 52347). This 2347-step gap is a complete blind spot.
3. Per-force-group energy decomposition. Since StateDataReporter reports only total energy, there is no record of which force term diverged.
4. The causal chain: did a single atom's velocity spike, causing a close contact, causing a force spike, causing a coordinate divergence, causing NaN? This temporal chain is entirely invisible.

### 6.5 Information Available to an External Observer

An external observer (e.g., ATHENA's Trace Semantics Engine) can reconstruct:

| Information | Source | Quality |
|:---|:---|:---|
| The simulation crashed due to NaN | Exception message or StateDataReporter output | Definitive |
| Energy was diverging before the crash | StateDataReporter: energy jump between step 50000 and 52000 | Definitive but coarse-grained |
| Coordinates were normal at step 50000 | DCDReporter last frame | Definitive |
| Force field used | System object (queryable) | Complete |
| Platform and precision | Context properties | Complete |
| Time step, temperature, constraints | Integrator properties | Complete |
| Which force term caused divergence | NOT AVAILABLE from default reporters | Missing |
| Exact step of failure onset | NOT AVAILABLE (between two reporting intervals) | Missing |
| Whether single or double precision would have prevented the failure | NOT AVAILABLE | Missing |
| Whether the root cause is a bad contact in the initial structure | Requires analysis of coordinates, not directly reported | Requires reconstruction |

### 6.6 Implications for IR Design

This walkthrough reveals that the default OpenMM trace output is **insufficient for deterministic fault isolation**. Specifically:

1. **Temporal resolution is too coarse.** The 1000-step gap between StateDataReporter entries creates blind spots where failures originate but are not captured.
2. **Energy decomposition is absent.** Total energy reveals *that* a problem exists but not *where* in the force field it originates.
3. **Crash-time state is unrecoverable.** The most valuable diagnostic information (the state at the moment of failure) is the state most likely to be lost.
4. **The theory-implementation boundary is blurred at the failure point.** A NaN energy can originate from bad force field parameters (theory), too-large time step (methodology), or single-precision overflow (implementation), and nothing in the default trace output distinguishes these.

An ATHENA-specific reporter would need to address all four gaps. See Section 5.4 for the proposed instrumentation strategy.

---

## 7. Failure Mode Taxonomy

### 7.1 Implementation Failures

**I1: NaN from single-precision overflow**
- **Mechanism:** Single-precision floating point on GPU has limited range (~1e38). Close atomic contacts produce repulsive forces exceeding this range, causing overflow to infinity, which propagates to NaN.
- **Signature:** Energy suddenly becomes NaN or infinity. Often occurs early in simulation (before equilibration resolves initial bad contacts). More common on CUDA/OpenCL with `Precision=single`.
- **Distinguishing test:** Run identical simulation on Reference platform (double precision). If NaN disappears, the cause is precision.
- **Evidence:** OpenMM documentation recommends `Precision=mixed` or `Precision=double` for initial energy minimization. Multiple GitHub issues (openmm/openmm#2143, forum discussions on NaN energies) document this pattern.

**I2: GPU memory exhaustion**
- **Mechanism:** System exceeds available GPU memory. Fails at Context creation or during PME grid allocation.
- **Signature:** `OpenMMException` at Context creation with memory-related error message.
- **Distinguishing test:** Same simulation succeeds with smaller system or on hardware with more memory.

**I3: CUDA/OpenCL driver incompatibility**
- **Mechanism:** Mismatch between OpenMM's CUDA kernels and the installed CUDA driver version.
- **Signature:** `OpenMMException` at Context creation referencing CUDA initialization failure.
- **Distinguishing test:** Same simulation succeeds on Reference or CPU platform.

**I4: Non-deterministic force summation artifacts**
- **Mechanism:** CUDA's default non-deterministic force summation (due to parallel reduction order) produces slightly different results across runs. Over long simulations, trajectories diverge.
- **Signature:** Not an error per se -- manifests as different trajectories from identical starting conditions. Only detectable by comparing runs or enabling `DeterministicForces=true`.
- **Distinguishing test:** Enable `DeterministicForces=true` and compare runs.
- **Evidence:** OpenMM documentation on Platform properties; `DeterministicForces` property documentation.

**I5: Checkpoint portability failures**
- **Mechanism:** Checkpoint binary format is platform-specific. Loading a CUDA checkpoint on the CPU platform (or a different GPU) fails or produces wrong results.
- **Signature:** Exception on checkpoint load, or silently corrupted state.
- **Distinguishing test:** Use portable state XML instead of checkpoint binary.
- **Evidence:** OpenMM User Guide, CheckpointReporter documentation explicitly warns about portability.

### 7.2 Methodology Failures

**M1: Insufficient energy minimization**
- **Mechanism:** Initial structure has steric clashes (overlapping atoms). Without adequate minimization, the first dynamics step produces enormous forces causing NaN.
- **Signature:** NaN within first ~100 steps of dynamics. Energy minimization (if performed) did not converge or was skipped.
- **Distinguishing test:** Perform more aggressive minimization. If NaN disappears, the cause was methodology.
- **Evidence:** Standard MD best practice, OpenMM tutorials always include `simulation.minimizeEnergy()` before dynamics.

**M2: Time step too large**
- **Mechanism:** Integration time step exceeds the Nyquist limit for the highest-frequency motions. Hydrogen vibrations (~10 fs period) require time steps below ~1 fs without constraints, or below ~2 fs with HBonds constraints. Exceeding this causes energy non-conservation.
- **Signature:** Gradual energy drift upward (NVE ensemble) or temperature instability (NVT ensemble). May eventually cause NaN.
- **Distinguishing test:** Reduce time step. If energy conservation improves, the cause was methodology.
- **Evidence:** MD textbook knowledge; OpenMM documentation on time steps and constraints.

**M3: Inadequate equilibration**
- **Mechanism:** Production data collected before the system reaches thermodynamic equilibrium. Observables show trends rather than fluctuations around a mean.
- **Signature:** Systematic drift in energy, density, or other observables during the "production" phase. NOT detected by OpenMM -- only visible in post-analysis.
- **Distinguishing test:** Extend equilibration and check for observable convergence.

**M4: Insufficient sampling**
- **Mechanism:** Simulation too short to sample relevant conformational states. Observables have not converged.
- **Signature:** Large statistical error bars, autocorrelation times exceeding the simulation length. NOT detected by OpenMM.
- **Distinguishing test:** Extend simulation or use enhanced sampling methods. Compute autocorrelation times.

**M5: Wrong ensemble choice**
- **Mechanism:** Using NVT when NPT is needed (or vice versa). For example, simulating a crystal at constant volume when the density is wrong.
- **Signature:** Pressure anomalies (NVT), density anomalies (NVT when system should be NPT). NOT detected by OpenMM as an error.
- **Distinguishing test:** Compare results across ensembles.

### 7.3 Theory Failures

**T1: Wrong force field for the system**
- **Mechanism:** Using a force field parameterized for a different class of molecules. For example, using a protein force field for a small organic molecule without proper parameterization.
- **Signature:** Incorrect structural properties (wrong bond lengths, angles), incorrect thermodynamic properties (wrong density, heat capacity), or NaN from extreme parameters. May be detected at `createSystem()` if atom types are missing.
- **Distinguishing test:** Compare simulation observables against experimental data. Use a different, appropriately parameterized force field.

**T2: Incorrect protonation states**
- **Mechanism:** Amino acid protonation states assigned incorrectly for the simulation pH. Histidine is the classic case (HIS can be HID, HIE, or HIP). Wrong protonation changes the electrostatic environment.
- **Signature:** Incorrect pKa values, wrong hydrogen bonding patterns, possibly NaN if protonation creates charge imbalance.
- **Distinguishing test:** Perform pKa calculations (e.g., PROPKA) and reassign protonation states.

**T3: Missing or incorrect force field parameters**
- **Mechanism:** Force field lacks parameters for a specific interaction (e.g., a novel ligand). The user provides incorrect custom parameters.
- **Signature:** If parameters are completely missing, `createSystem()` throws an exception (detectable). If parameters are present but wrong, the simulation runs but produces incorrect physics (undetectable by OpenMM).
- **Distinguishing test:** Compare computed properties against quantum mechanical reference calculations or experimental data.

**T4: Incorrect system setup (missing solvent, ions, cofactors)**
- **Mechanism:** The simulation system is missing critical components. For example, simulating a metalloprotein without the metal ion, or an enzyme without its cofactor.
- **Signature:** Structural collapse, incorrect dynamics, or subtle errors in binding energetics. NOT detected by OpenMM.
- **Distinguishing test:** Validate system composition against experimental structure.

### 7.4 Ambiguous Failures

**A1: NaN energy without clear origin**
- **Mechanism:** NaN in energy can arise from implementation (precision), methodology (time step, bad initial structure), or theory (extreme force field parameters). The default trace output does not distinguish these.
- **Resolution requires:** Per-force-group energy decomposition, precision comparison (Reference vs. CUDA), time step sensitivity analysis, and initial structure validation -- none of which are automated.

**A2: Constraint failure during dynamics**
- **Mechanism:** SHAKE/SETTLE constraint solver fails to converge. Can be caused by: too-large time step (methodology), bad initial structure (methodology), incompatible constraints (theory), or accumulated numerical error (implementation).
- **Signature:** `OpenMMException` referencing constraint tolerance. The exception message does not indicate the root cause category.
- **Resolution requires:** Analyzing which constraint failed, whether the time step is appropriate, and whether the constrained geometry is physically reasonable.

**A3: Energy drift in NVE ensemble**
- **Mechanism:** Total energy should be conserved in NVE (microcanonical) ensemble. Drift can be caused by: integration errors from time step (methodology), numerical precision (implementation), or PME parameter choices (straddling theory/implementation).
- **Resolution requires:** Time step convergence test, precision comparison, PME parameter sensitivity analysis.

### 7.5 Taxonomy Summary

| ID | Failure Mode | Category | Detected by OpenMM? | Distinguishable from default trace? |
|:---|:---|:---|:---|:---|
| I1 | Single-precision overflow | Implementation | Sometimes (NaN) | No -- requires precision comparison |
| I2 | GPU memory exhaustion | Implementation | Yes (exception) | Yes |
| I3 | Driver incompatibility | Implementation | Yes (exception) | Yes |
| I4 | Non-deterministic summation | Implementation | No | No |
| I5 | Checkpoint portability | Implementation | Sometimes | Partially |
| M1 | Insufficient minimization | Methodology | No | No |
| M2 | Time step too large | Methodology | No | No (gradual drift) |
| M3 | Inadequate equilibration | Methodology | No | No |
| M4 | Insufficient sampling | Methodology | No | No |
| M5 | Wrong ensemble | Methodology | No | No |
| T1 | Wrong force field | Theory | Sometimes (at createSystem) | Partially |
| T2 | Wrong protonation | Theory | No | No |
| T3 | Bad parameters | Theory | Sometimes (at createSystem) | Partially |
| T4 | Missing components | Theory | No | No |
| A1 | NaN (ambiguous origin) | Ambiguous | Sometimes | No |
| A2 | Constraint failure | Ambiguous | Yes (exception) | Partially |
| A3 | NVE energy drift | Ambiguous | No | No |

**Key finding:** Of 17 cataloged failure modes, only 4 are definitively detectable and classifiable from default OpenMM trace output (I2, I3, and partially T1/T3 when they cause createSystem exceptions). The remaining 13 either go undetected or are detected without sufficient information to classify them. This validates the need for an enhanced trace capture system as a prerequisite for the LFI's three-stage audit.

---

## 8. Conclusions for IR Design

### 8.1 The Good News

OpenMM has a **clean theory-implementation API boundary** that maps well to ATHENA's architectural requirements:
- The `ForceField` / `Topology` / `System` chain is a well-defined theory specification pipeline.
- The `Platform` / `Context` separation cleanly isolates implementation choices.
- The `ForceField.createSystem()` boundary is an explicit compilation step that can be audited.
- The reporter API is extensible enough to support custom trace capture without modifying OpenMM's core.
- Per-force-group energy decomposition is available via the API (just not used by default reporters).

### 8.2 The Bad News

OpenMM's **default trace output is sparse and temporally coarse**:
- Reporters fire at fixed intervals, creating blind spots.
- Total energy is reported, but not energy decomposition by force term.
- Crash-time state is often unrecoverable.
- Methodology failures are invisible to the framework.
- The most common failure (NaN energy) is ambiguous between all three categories without additional diagnostic instrumentation.

### 8.3 Implications for the IR

1. **The IR must operate on enhanced traces, not default traces.** A custom ATHENA reporter is a prerequisite for effective fault isolation in OpenMM.
2. **The IR must represent the theory-implementation boundary explicitly.** The ForceField -> createSystem() -> System -> Context chain defines natural layers that the IR should preserve.
3. **The IR must handle temporal gaps.** Between reporting intervals, the IR must represent "unknown state" rather than interpolating. The gap itself is diagnostic information (a crash in a gap region means pre-crash state is unavailable).
4. **The IR needs a per-force-term energy decomposition representation.** The most valuable diagnostic for NaN/explosion failures is knowing which force group diverged first.
5. **Methodology-layer failures require external criteria.** OpenMM provides no methodology assessment. The IR must incorporate domain-specific methodology rules (equilibration criteria, sampling sufficiency) as external annotations, not as parsed trace data.

### 8.4 Comparison Baseline for Other DSLs

This analysis establishes OpenMM as the first DSL surveyed. When GROMACS and VASP are surveyed, the key comparison points will be:
- Does the DSL enforce a comparably clean theory-implementation separation?
- Does the DSL provide richer default trace output (especially energy decomposition)?
- Does the DSL surface methodology-layer diagnostics?
- Does the DSL provide crash-time state recovery?
- Is the DSL's trace output extensible?

---

**Sources cited in this document:**
- OpenMM Python API documentation: docs.openmm.org/latest/api-python/
- OpenMM User Guide: openmm.org/documentation (Chapters 3, 4, 8)
- OpenMM source code: github.com/openmm/openmm (wrappers/python/openmm/app/)
  - `simulation.py` (reporter invocation logic)
  - `statedatareporter.py` (StateDataReporter implementation)
  - `dcdreporter.py` (DCDReporter implementation)
  - `pdbreporter.py` (PDBReporter implementation)
  - `pdbxreporter.py` (PDBxReporter implementation)
  - `checkpointreporter.py` (CheckpointReporter implementation)
  - `forcefield.py` (ForceField.createSystem implementation)
  - `topology.py` (Topology data structures)
- OpenMM GitHub Issues: openmm/openmm issues on NaN energy, precision, constraint failures
- mdtraj documentation: mdtraj.org (XTCReporter, HDF5Reporter)
- OpenMMTools documentation: openmmtools.readthedocs.io
