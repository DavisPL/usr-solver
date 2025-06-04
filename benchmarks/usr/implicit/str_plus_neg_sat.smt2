(set-logic QF_S)

(declare-const s1 String)
(declare-const s2 String)

(assert (str.in_re s2 (re.++ (str.to_re "el") (str.to_re "lo") )))

(assert (not (= (str.++ s1 s2) "hello")))

(check-sat)

(get-model)

