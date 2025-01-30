;regexA is same regex as in passw_sat1.smt2
(declare-const regexA RegLan)
(define-fun Witness () String (str.++ (str.++ "a" "A") "0"))
(assert (= regexA (re.inter (re.inter (re.inter (re.++ (re.++ re.all (re.range "a" "z")) re.all) (re.++ (re.++ re.all (re.range "A" "Z")) re.all)) (re.++ (re.++ re.all (re.range "0" "9")) re.all)) ((_ re.loop 0 3) (re.range "!" "~")))))
(assert (str.in_re Witness regexA))
(check-sat)
