# Benchmarks to investigate

### USR

Crash for `usr_a`:
```
benchmarks/usr/explicit/usr_8_sat.smt2
benchmarks/usr/explicit/index_of_neg_unsat.smt2
benchmarks/usr/implicit/str_at_neg_sat.smt2
benchmarks/usr/implicit/index_of_sat.smt2
benchmarks/usr/implicit/index_of_neg_unsat.smt2
benchmarks/usr/implicit/str_at_sat.smt2
benchmarks/usr/implicit/str_cmp_sat.smt2
```

Timeouts for `usr_a`:

```
benchmarks/usr/explicit/simple_usr_unsat_5.smt2
benchmarks/usr/explicit/usr_10_unsat.smt2
benchmarks/usr/explicit/cats_unsat.smt2
benchmarks/usr/explicit/substr_unsat.smt2
benchmarks/usr/implicit/substr_sat.smt2
benchmarks/usr/implicit/substr_sat_minimized.smt2
benchmarks/usr/implicit/substr_unsat.smt2
```

### SMT and Regex Benchmarks

A wrong case, looks very simple/innocent! - interesting
```
benchmarks/from_dz3-artifact/QF_SLIA_Norn/ab/norn-benchmark-102.smt2
```

Other wrong answers for USR
```
benchmarks/regex-smt-benchmarks-main/regexlib_subset/unsat/notsubset_1_1.smt2
benchmarks/regex-smt-benchmarks-main/regexlib_subset/unsat/notsubset_3_3.smt2
benchmarks/from_dz3-artifact/QF_SLIA_Norn/ab/norn-benchmark-102.smt2
benchmarks/from_dz3-artifact/QF_SLIA_Norn/ab/norn-benchmark-112.smt2
benchmarks/from_dz3-artifact/QF_SLIA_Norn/ab/norn-benchmark-116.smt2
benchmarks/from_dz3-artifact/QF_SLIA_Norn/ab/norn-benchmark-31.smt2
benchmarks/from_dz3-artifact/QF_SLIA_Norn/HammingDistance/norn-benchmark-1049.smt2
benchmarks/from_dz3-artifact/QF_SLIA_Norn/HammingDistance/norn-benchmark-235.smt2
benchmarks/from_dz3-artifact/QF_SLIA_Norn/HammingDistance/norn-benchmark-378.smt2
benchmarks/from_dz3-artifact/QF_SLIA_Norn/HammingDistance/norn-benchmark-489.smt2
benchmarks/from_dz3-artifact/QF_SLIA_Norn/HammingDistance/norn-benchmark-585.smt2
benchmarks/from_dz3-artifact/QF_SLIA_Norn/HammingDistance/norn-benchmark-599.smt2
```
